// SPDX-License-Identifier: Apache-2.0
//! Comments: found outside strings, attached to the model by position,
//! written back by the canonical layout.
use colap::config_model::{ConfigModel, ConfigValue, EntityNode, NodeId};
use colap::format::{format_file, format_str};

fn model(path: &str) -> ConfigModel {
    colap::parse_file(path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .model
}

fn with<T>(model: &ConfigModel, path: &str, f: impl FnOnce(&EntityNode) -> T) -> T {
    let id: NodeId = model
        .find_entity_by_path(path)
        .unwrap_or_else(|| panic!("no entity `{path}`"));
    let node = model.get_node(id).expect("node");
    let node = node.borrow();
    f(&node)
}

fn leading(model: &ConfigModel, path: &str) -> Vec<String> {
    with(model, path, |n| n.comments.leading.clone())
}

#[test]
fn every_placement_attaches_where_the_spec_says() {
    let m = model("tests/conformance/valid/comments.md");
    assert_eq!(leading(&m, "setting"), vec![" Leads the entity below."]);
    with(&m, "setting", |n| {
        assert_eq!(n.comments.trailing.as_deref(), Some(" trails the header"));
        assert!(n.closing.is_empty());
    });
    with(&m, "setting/verbose", |n| {
        assert_eq!(n.comments.leading, vec![" leads `verbose`"]);
        assert_eq!(
            n.closing.trailing.as_deref(),
            Some(" trails the one-line entity's `;`")
        );
    });
    with(&m, "setting/url", |n| {
        assert!(n.comments.is_empty());
        assert_eq!(
            n.fields[0].comments.trailing.as_deref(),
            Some(" trails the field")
        );
        assert_eq!(
            n.fields[1].comments.trailing.as_deref(),
            Some(" trails the block string's field")
        );
        assert_eq!(
            n.fields[2].comments.leading,
            vec![" inside a list: leads the field after formatting"]
        );
        assert!(n.fields[2].comments.trailing.is_none());
        assert_eq!(n.closing.leading, vec![" leads the close of `url`"]);
        assert_eq!(
            n.closing.trailing.as_deref(),
            Some(" trails the close of `url`")
        );
    });
    // Block ends, across both blocks, on the root.
    let root = m.get_node(m.root_id()).expect("root");
    assert_eq!(
        root.borrow().closing.leading,
        vec![" Ends the block.", " Nothing here."]
    );
}

#[test]
fn a_double_slash_inside_a_string_is_text() {
    let m = model("tests/conformance/valid/comments.md");
    let id = m.find_entity_by_path("setting/url").expect("url");
    match m.get_field_value(id, "value") {
        Some(ConfigValue::String(s)) => assert_eq!(s, "http://example.org/not-a-comment"),
        other => panic!("{other:?}"),
    }
    match m.get_field_value(id, "gloss") {
        Some(ConfigValue::String(s)) => {
            assert_eq!(s, "A // inside a block string is text too.")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn fmt_keeps_every_comment_and_is_idempotent() {
    let formatted = format_file("tests/data/fmt/comments.md").expect("format");
    let expected = std::fs::read_to_string("tests/data/fmt/comments.expected.md").expect("read");
    assert_eq!(formatted.text, expected);
    assert_eq!(
        formatted.changed,
        vec![3],
        "the first block; the second is canonical"
    );
    let source = std::fs::read_to_string("tests/data/fmt/comments.md").expect("read");
    for line in source.lines().filter(|l| l.contains("//")) {
        let comment = &line[line.rfind("//").expect("marker")..];
        assert!(expected.contains(comment), "lost: {comment}");
    }
}

#[test]
fn bare_cola_with_comments_formats_too() {
    let text = "// head\na:   x: 1 ,  // one\n  y: 2;\n// tail\n";
    let formatted = format_str(text, None).expect("format");
    assert_eq!(
        formatted.text,
        "// head\na:\n    x: 1, // one\n    y: 2,\n;\n// tail\n"
    );
    let again = format_str(&formatted.text, None).expect("format");
    assert!(again.is_canonical());
}
