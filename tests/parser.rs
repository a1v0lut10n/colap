// SPDX-License-Identifier: Apache-2.0
//! Integration tests for the Cola parser using Rustemo
use rustemo::Parser;
use colap::cola::ColaParser;
use colap::cola_actions::Cola;
use std::fs;
use std::path::Path;

/// Helper to parse a file and assert success
fn parse_cola_file<P: AsRef<Path>>(path: P) -> Cola {
    let input = fs::read_to_string(&path).expect(&format!(
        "Failed to read test file: {}",
        path.as_ref().display()
    ));
    let parser = ColaParser::new();
    parser
        .parse(&input)
        .expect(&format!("Parse failed for {}", path.as_ref().display()))
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
