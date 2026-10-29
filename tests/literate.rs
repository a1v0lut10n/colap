// SPDX-License-Identifier: Apache-2.0
//! The markdown layer: fences of every shape, prose of every shape,
//! locations in the original file, dialect detection, and the editor
//! highlighter agreeing with the scanner. Fixtures live under
//! `tests/data/literate/`; the reproductions come from the 2026-10-29
//! field-test tasks.
use colap::config_model::ConfigValue;
use colap::highlight::{LineState, line_tokens};
use colap::literate::{Warning, scan};
use colap::{ConfigModel, Error, Parsed};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new("tests/data/literate").join(name)
}

fn parse(name: &str) -> Parsed {
    colap::parse_file(fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn top_level_names(model: &ConfigModel) -> Vec<String> {
    let root = model.get_node(model.root_id()).expect("root");
    let root = root.borrow();
    root.children
        .iter()
        .filter_map(|&id| model.get_node(id))
        .map(|n| n.borrow().name.clone())
        .collect()
}

fn field_int(model: &ConfigModel, path: &str, field: &str) -> Option<i64> {
    let id = model.find_entity_by_path(path)?;
    match model.get_field_value(id, field)? {
        ConfigValue::Integer(i) => Some(i),
        other => panic!("{path}.{field} is {other:?}"),
    }
}

#[test]
fn foreign_fences_are_skipped_and_cola_blocks_around_them_parse() {
    let parsed = parse("foreign-fences.md");
    assert!(parsed.warnings.is_empty());
    assert_eq!(top_level_names(&parsed.model), ["a", "b", "c"]);
    assert_eq!(field_int(&parsed.model, "a", "x"), Some(1));
    assert_eq!(field_int(&parsed.model, "b", "y"), Some(2));
    assert_eq!(field_int(&parsed.model, "c", "z"), Some(3));
    // The ```text block that looks like cola never reaches the model.
    assert!(parsed.model.find_entity_by_path("looks").is_none());
}

#[test]
fn any_prose_line_outside_a_fence_is_fine() {
    let parsed = parse("prose-lines.md");
    assert!(parsed.warnings.is_empty());
    assert_eq!(top_level_names(&parsed.model), ["p"]);
}

#[test]
fn tilde_long_indented_and_info_string_fences() {
    let parsed = parse("tilde-and-long-fences.md");
    assert!(parsed.warnings.is_empty());
    assert_eq!(top_level_names(&parsed.model), ["t", "i", "indented"]);
    // The ```cola example inside the ````md block is quoted, not parsed.
    assert!(parsed.model.find_entity_by_path("quoted").is_none());
}

#[test]
fn language_tags_with_digits_and_punctuation() {
    let parsed = parse("language-tags.md");
    assert_eq!(top_level_names(&parsed.model), ["l"]);
}

#[test]
fn closing_fence_at_end_of_file_without_newline() {
    let parsed = parse("eof-no-newline.md");
    assert!(parsed.warnings.is_empty());
    assert_eq!(field_int(&parsed.model, "e", "f"), Some(1));
}

#[test]
fn unterminated_fence_warns_and_still_parses() {
    let parsed = parse("unterminated.md");
    assert_eq!(
        parsed.warnings,
        vec![Warning::UnterminatedFence {
            line: 3,
            lang: "cola".into()
        }]
    );
    assert_eq!(top_level_names(&parsed.model), ["u"]);
}

#[test]
fn locations_point_into_the_original_file() {
    let path = fixture("foreign-fences.md");
    let parsed = parse("foreign-fences.md");
    let b = parsed.model.find_entity_by_path("b").expect("b");
    let node = parsed.model.get_node(b).expect("node");
    let b = node.borrow();
    let loc = b.location.as_ref().expect("entity location");
    assert_eq!(loc.file_path, path);
    assert_eq!(loc.start_line, 29, "`b:` is on line 29 of the file");

    // Fields carry their own location.
    let y = b
        .field("y")
        .and_then(|f| f.location.clone())
        .expect("field y location");
    assert_eq!((y.start_line, y.start_column), (30, 4));
}

#[test]
fn an_error_in_the_second_block_reports_the_files_own_line() {
    let path = fixture("second-block-error.md");
    let err = colap::parse_file(&path).expect_err("the second block is broken");
    match &err {
        Error::Parse {
            path: p,
            line,
            column,
            message,
        } => {
            assert_eq!(p.as_deref(), Some(path.as_path()));
            assert_eq!((*line, *column), (13, 8));
            assert!(message.starts_with("Expected one of"), "{message}");
        }
        other => panic!("not a parse error: {other}"),
    }
    assert!(
        err.to_string()
            .starts_with(&format!("{}:13:8: Expected one of", path.display())),
        "{err}"
    );
}

#[test]
fn bare_cola_files_and_literate_cola_files_both_parse() {
    let bare = parse("bare.cola");
    assert_eq!(top_level_names(&bare.model), ["root_entity"]);
    let lit = parse("literate.cola");
    assert_eq!(top_level_names(&lit.model), ["lit"]);
}

#[test]
fn markdown_without_cola_is_empty_when_literate_and_an_error_when_detected() {
    let text = std::fs::read_to_string(fixture("no-cola.md")).unwrap();
    let parsed = colap::parse_literate_str(&text, None).expect("literate parse");
    assert!(top_level_names(&parsed.model).is_empty());
    assert!(parsed.warnings.is_empty());
    // `.md` goes the literate way through parse_file, too.
    assert!(top_level_names(&parse("no-cola.md").model).is_empty());
    // Detection treats fence-less text as bare cola: prose is not cola.
    let err = colap::parse_str(&text, None).expect_err("prose is not cola");
    assert!(matches!(err, Error::Parse { line: 1, .. }), "{err}");
}

#[test]
fn parse_str_detects_the_dialect() {
    let bare = colap::parse_str("a:\n  b: 1\n;\n", None).unwrap();
    assert_eq!(field_int(&bare.model, "a", "b"), Some(1));
    let lit = colap::parse_str("# T\n\n```cola\na:\n  b: 2\n;\n```\n", None).unwrap();
    assert_eq!(field_int(&lit.model, "a", "b"), Some(2));
}

/// The lines the scanner hands to the grammar, 1-based.
fn cola_lines_by_scanner(text: &str) -> BTreeSet<u32> {
    scan(text)
        .cola_blocks()
        .flat_map(|b| {
            let n = b.body.split_inclusive('\n').count() as u32;
            b.body_line()..b.body_line() + n
        })
        .collect()
}

/// The lines the highlighter lexes as cola, 1-based.
fn cola_lines_by_highlighter(text: &str) -> BTreeSet<u32> {
    let mut state = LineState::default();
    let mut out = BTreeSet::new();
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let (_, next) = line_tokens(line, state);
        if state.is_cola() && next.is_cola() {
            out.insert(index as u32 + 1);
        }
        state = next;
    }
    out
}

#[test]
fn highlighter_and_scanner_agree_on_every_fixture() {
    let mut checked = 0;
    for dir in ["tests/data", "tests/data/literate"] {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            assert_eq!(
                cola_lines_by_highlighter(&text),
                cola_lines_by_scanner(&text),
                "{}",
                path.display()
            );
            checked += 1;
        }
    }
    assert!(checked >= 15, "only {checked} fixtures checked");
}
