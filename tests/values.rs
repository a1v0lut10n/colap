// SPDX-License-Identifier: Apache-2.0
//! The value language: what each literal form becomes in the model,
//! per `docs/design/2026-10-30-value-language.md`.
use colap::config_model::{ConfigModel, ConfigValue};

fn model(name: &str) -> ConfigModel {
    colap::parse_file(format!("tests/conformance/valid/{name}"))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
        .model
}

fn value(model: &ConfigModel, path: &str, field: &str) -> ConfigValue {
    let id = model
        .find_entity_by_path(path)
        .unwrap_or_else(|| panic!("no entity `{path}`"));
    model
        .get_field_value(id, field)
        .unwrap_or_else(|| panic!("no field `{field}` on `{path}`"))
}

fn text(model: &ConfigModel, path: &str, field: &str) -> String {
    value(model, path, field).to_string()
}

#[test]
fn strings_decode_escapes_and_blocks_dedent() {
    let m = model("values.md");
    let string = |field: &str| match value(&m, "sample", field) {
        ConfigValue::String(s) => s,
        other => panic!("{field} is {other:?}"),
    };
    assert_eq!(string("double"), "say \"hi\"");
    assert_eq!(string("single"), "it's");
    assert_eq!(string("escapes"), "tab\tnewline\nbackslash\\unicodeé");
    assert_eq!(string("multi"), "two\nlines");
    assert_eq!(string("block"), "A gloss over\n  two lines, one indented.");
}

#[test]
fn numbers_take_separators_and_exponents() {
    let m = model("values.md");
    assert_eq!(text(&m, "sample", "grouped"), "1000000");
    assert_eq!(text(&m, "sample", "padded"), "7");
    assert_eq!(text(&m, "sample", "exponent"), "0.0015");
    assert_eq!(text(&m, "sample", "big"), "20000000000.0");
    assert!(matches!(value(&m, "sample", "big"), ConfigValue::Float(_)));
    assert!(matches!(
        value(&m, "sample", "grouped"),
        ConfigValue::Integer(_)
    ));
}

#[test]
fn symbols_lists_and_references() {
    let m = model("symbols-lists-references.md");
    assert!(matches!(value(&m, "claims/cl-1", "basis"), ConfigValue::Symbol(s) if s == "parsed"));
    assert!(matches!(value(&m, "claims/cl-1", "shape"), ConfigValue::Symbol(s) if s == "plural"));
    assert!(matches!(
        value(&m, "claims/cl-1", "supports"),
        ConfigValue::Reference(p) if p == "claims/cl-2"
    ));
    assert_eq!(
        text(&m, "claims/cl-1", "tags"),
        "[alpha, \"two words\", 3, 1.5, true, @claims/cl-2]"
    );
    assert!(
        matches!(value(&m, "claims/cl-1", "none"), ConfigValue::List(items) if items.is_empty())
    );
    assert_eq!(text(&m, "claims/cl-2", "tags"), "[beta]");
    assert_eq!(
        text(&m, "claims/cl-2/detail", "note"),
        "@claims/cl-2/detail"
    );
}

#[test]
fn unresolved_references_are_errors_with_the_field_location() {
    let err = colap::parse_literate_str(
        "```cola\nlink plural links:\n    l-1: to: @links/l-2;\n;\n```\n",
        None,
    )
    .expect_err("unresolved");
    assert_eq!(err.to_string(), "3:10: unresolved reference `@links/l-2`");
}

#[test]
fn trailing_commas_do_not_change_the_model() {
    let with = colap::parse_cola_str("a: x: 1, y: [1, 2,], inner: z: 3,; w: 4,;", None)
        .expect("trailing commas")
        .model;
    let without = colap::parse_cola_str("a: x: 1, y: [1, 2] inner: z: 3; w: 4;", None)
        .expect("plain")
        .model;
    assert_eq!(with.to_cola(), without.to_cola());
    assert!(colap::parse_cola_str("a: x: 1,, y: 2;", None).is_err());
    assert!(colap::parse_cola_str("a: inner: z: 3;, y: 2;", None).is_err());
    assert!(colap::parse_cola_str("a: y: [,];", None).is_err());
}
