// SPDX-License-Identifier: Apache-2.0
//! The duplicate rules of `docs/design/2026-10-29-duplicate-key-semantics.md`:
//! keys are defined once, slots reopen, fields are set once, one
//! namespace per entity, every error reported. Fixtures live under
//! `tests/conformance/` (valid and invalid).
use colap::config_model::{ConfigModel, ConfigValue, DuplicateKind, ModelError};
use colap::{Error, Parsed};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    // The fixtures are the conformance corpus: a file is valid or invalid.
    let valid = Path::new("tests/conformance/valid").join(name);
    if valid.exists() {
        valid
    } else {
        Path::new("tests/conformance/invalid").join(name)
    }
}

fn parse(name: &str) -> Parsed {
    colap::parse_file(fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn fail(name: &str) -> Error {
    colap::parse_file(fixture(name))
        .err()
        .unwrap_or_else(|| panic!("{name} should not parse"))
}

fn child_names(model: &ConfigModel, path: &str) -> Vec<String> {
    let id = model
        .find_entity_by_path(path)
        .unwrap_or_else(|| panic!("no `{path}`"));
    let node = model.get_node(id).expect("node");
    let node = node.borrow();
    node.children
        .iter()
        .filter_map(|&id| model.get_node(id))
        .map(|n| n.borrow().name.clone())
        .collect()
}

fn field_names(model: &ConfigModel, path: &str) -> Vec<String> {
    let id = model
        .find_entity_by_path(path)
        .unwrap_or_else(|| panic!("no `{path}`"));
    let node = model.get_node(id).expect("node");
    let node = node.borrow();
    node.fields.iter().map(|f| f.name.clone()).collect()
}

fn int(model: &ConfigModel, path: &str, field: &str) -> Option<i64> {
    let id = model.find_entity_by_path(path)?;
    match model.get_field_value(id, field)? {
        ConfigValue::Integer(i) => Some(i),
        other => panic!("{path}.{field} is {other:?}"),
    }
}

fn start_line(model: &ConfigModel, path: &str) -> u32 {
    let id = model
        .find_entity_by_path(path)
        .unwrap_or_else(|| panic!("no `{path}`"));
    let node = model.get_node(id).expect("node");
    let node = node.borrow();
    node.location.as_ref().expect("location").start_line
}

/// The error is one located duplicate of `kind`, at `line:column` of
/// the fixture, displayed exactly as `message` after the location.
fn expect_duplicate(
    err: &Error,
    name: &str,
    kind: DuplicateKind,
    line: u32,
    column: u32,
    message: &str,
) {
    match err {
        Error::Model { path, error } if matches!(**error, ModelError::Duplicate(_)) => {
            let ModelError::Duplicate(d) = error.as_ref() else {
                unreachable!()
            };
            assert_eq!(path.as_deref(), Some(fixture(name).as_path()));
            assert_eq!(d.kind, kind, "{err}");
            assert_eq!(
                d.second
                    .as_ref()
                    .map(|l| (l.start_line, l.start_column + 1)),
                Some((line, column)),
                "{err}"
            );
        }
        other => panic!("{name}: not a duplicate: {other}"),
    }
    assert_eq!(
        err.to_string(),
        format!("{}:{line}:{column}: {message}", fixture(name).display())
    );
}

#[test]
fn a_field_set_twice_is_an_error() {
    let err = fail("field-twice.md");
    expect_duplicate(
        &err,
        "field-twice.md",
        DuplicateKind::Field,
        6,
        5,
        "duplicate field `x` in `a`; first at 5:5",
    );
}

#[test]
fn a_key_defined_twice_in_one_block_is_an_error() {
    let err = fail("key-twice-one-block.md");
    expect_duplicate(
        &err,
        "key-twice-one-block.md",
        DuplicateKind::Key,
        8,
        5,
        "duplicate entry `cl-1` in `claim`; first at 5:5",
    );
}

#[test]
fn a_key_defined_twice_across_blocks_is_an_error() {
    let err = fail("key-twice-across-blocks.md");
    expect_duplicate(
        &err,
        "key-twice-across-blocks.md",
        DuplicateKind::Key,
        18,
        5,
        "duplicate entry `cl-1` in `claim`; first at 5:5",
    );
}

#[test]
fn a_plural_reopened_across_blocks_extends_it() {
    let parsed = parse("plural-extends.md");
    let model = &parsed.model;
    assert_eq!(child_names(model, ""), ["claim"], "one collection, not two");
    assert_eq!(child_names(model, "claim"), ["cl-1", "cl-2", "cl-3"]);
    assert_eq!(int(model, "claim/cl-3", "v"), Some(3));
    assert_eq!(
        start_line(model, "claim"),
        4,
        "the first opening is the location"
    );
    assert_eq!(start_line(model, "claim/cl-2"), 15);
}

#[test]
fn a_singular_reopened_adds_fields_and_slots() {
    let parsed = parse("singular-reopens.md");
    let model = &parsed.model;
    assert_eq!(child_names(model, ""), ["llm"]);
    assert_eq!(field_names(model, "llm"), ["backend", "timeout"]);
    assert_eq!(child_names(model, "llm"), ["provider"]);
    assert_eq!(
        model
            .get_field_value(
                model.find_entity_by_path("llm/provider/google").unwrap(),
                "key"
            )
            .map(|v| v.to_string()),
        Some("\"k\"".into())
    );
    assert_eq!(start_line(model, "llm"), 4);
}

#[test]
fn reopening_with_a_different_plural_name_is_an_error() {
    let err = fail("plural-name-mismatch.md");
    expect_duplicate(
        &err,
        "plural-name-mismatch.md",
        DuplicateKind::Declaration,
        12,
        1,
        "`model` in the root is declared `model plural models` and cannot be reopened as `model plural modelz`; first at 4:1",
    );
}

#[test]
fn reopening_a_singular_as_a_plural_is_an_error() {
    let err = fail("kind-mismatch.md");
    expect_duplicate(
        &err,
        "kind-mismatch.md",
        DuplicateKind::Declaration,
        7,
        1,
        "`api` in the root is declared `api` and cannot be reopened as `api plural apis`; first at 4:1",
    );
}

#[test]
fn a_name_is_a_field_or_an_entity_not_both() {
    let err = fail("field-vs-entity.md");
    expect_duplicate(
        &err,
        "field-vs-entity.md",
        DuplicateKind::Name,
        6,
        5,
        "`b` in `a` is already a field and cannot also be an entity; first at 5:5",
    );
}

#[test]
fn a_plural_name_cannot_be_taken() {
    let err = fail("plural-name-taken.md");
    expect_duplicate(
        &err,
        "plural-name-taken.md",
        DuplicateKind::Name,
        6,
        5,
        "`models` in `llm` is already taken and cannot be the plural name of `model`; first at 5:5",
    );
}

/// The consequence the design record calls out: a key cannot be
/// reopened to batch a nested collection under it.
#[test]
fn reopening_a_key_to_batch_under_it_is_an_error() {
    let err = fail("key-reopened.md");
    expect_duplicate(
        &err,
        "key-reopened.md",
        DuplicateKind::Key,
        19,
        5,
        "duplicate entry `openai` in `llm`; first at 5:5",
    );
}

#[test]
fn every_error_is_reported_in_line_order() {
    let err = fail("several-errors.md");
    let path = fixture("several-errors.md");
    let Error::Multiple(errors) = &err else {
        panic!("expected several errors, got one: {err}")
    };
    assert_eq!(errors.len(), 3);
    assert_eq!(err.each().count(), 3);
    assert_eq!(
        errors.iter().map(Error::line).collect::<Vec<_>>(),
        [Some(6), Some(12), Some(21)]
    );
    assert!(matches!(errors[1], Error::Parse { .. }), "{}", errors[1]);
    let expected = format!(
        "{p}:6:5: duplicate field `x` in `a`; first at 5:5\n\
         {p}:12:8: Expected one of BooleanFalse, BooleanTrue, LBracket, Semicolon, Identifier, Number, Reference, QuotedStringDouble, QuotedStringSingle, BlockString.\n\
         {p}:21:5: duplicate entry `cl-1` in `claim`; first at 18:5",
        p = path.display()
    );
    assert_eq!(err.to_string(), expected);
}

#[test]
fn the_model_prints_each_field_once() {
    let parsed = parse("plural-extends.md");
    let plain = parsed.model.to_string();
    let pretty = parsed.model.pretty_display();
    assert_eq!(plain.matches("v: 1").count(), 1, "{plain}");
    assert_eq!(pretty.matches("v: 1").count(), 1, "{pretty}");
    // Entries print in source order across the two blocks.
    let order: Vec<usize> = ["cl-1", "cl-2", "cl-3"]
        .iter()
        .map(|k| pretty.find(k).unwrap())
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{pretty}");
}
