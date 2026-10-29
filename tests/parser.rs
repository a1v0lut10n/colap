// SPDX-License-Identifier: Apache-2.0
//! Integration tests for the Cola parser: every fixture under
//! `tests/data/` parses through the facade, and the raw rustemo parser
//! still accepts a bare block.
use colap::cola::ColaParser;
use colap::rustemo::Parser;
use std::path::Path;

/// Helper to parse a file and assert success
fn parse_cola_file<P: AsRef<Path>>(path: P) -> colap::Parsed {
    colap::parse_file(&path)
        .unwrap_or_else(|e| panic!("Parse failed for {}: {e}", path.as_ref().display()))
}

#[test]
fn test_correct_md() {
    parse_cola_file("tests/data/test_correct.md");
}

#[test]
fn test_minimal_md() {
    parse_cola_file("tests/data/test_minimal.md");
}

#[test]
fn test_minimal_blockbody_md() {
    parse_cola_file("tests/data/test_minimal_blockbody.md");
}

#[test]
fn test_nested_md() {
    parse_cola_file("tests/data/test_nested.md");
}

#[test]
fn test_nested_oneline_md() {
    parse_cola_file("tests/data/test_nested_oneline.md");
}

#[test]
fn test_no_indent_md() {
    parse_cola_file("tests/data/test_no_indent.md");
}

#[test]
fn test_real_syntax_md() {
    parse_cola_file("tests/data/test_real_syntax.md");
}

#[test]
fn test_simple_md() {
    parse_cola_file("tests/data/test_simple.md");
}

#[test]
fn test_simple_cola_md() {
    parse_cola_file("tests/data/test_simple_cola.md");
}

#[test]
fn test_genite_md() {
    parse_cola_file("tests/data/test_genite.md");
}

#[test]
fn raw_parser_accepts_a_bare_block() {
    let ast = ColaParser::new()
        .parse("llm plural llms: openai: field: \"value\";;")
        .expect("bare cola parses");
    assert_eq!(ast.map(|entities| entities.len()), Some(1));
}
