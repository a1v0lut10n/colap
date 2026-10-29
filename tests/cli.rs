// SPDX-License-Identifier: Apache-2.0
//! The `colap` binary: exit codes, message lines, output formats, the
//! trace default. Runs the built executable on the fixtures.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn colap(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_colap"))
        .args(args)
        .env_remove("RUSTEMO_NOTRACE")
        .output()
        .expect("run colap")
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("colap-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn check_a_clean_file_is_silent_and_exits_0() {
    let out = colap(&["check", "tests/data/test_genite.md"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), "", "the trace must be off by default");
}

#[test]
fn check_reports_every_error_and_exits_1() {
    let path = "tests/data/duplicates/several-errors.md";
    let out = colap(&["check", path]);
    assert_eq!(code(&out), 1);
    assert_eq!(
        stdout(&out),
        format!(
            "{path}:6:5: error: duplicate field `x` in `a`; first at 5:5\n\
             {path}:12:8: error: Expected one of BooleanFalse, BooleanTrue, Semicolon, Identifier, Number, QuotedStringDouble, QuotedStringSingle.\n\
             {path}:21:5: error: duplicate entry `cl-1` in `claim`; first at 18:5\n"
        )
    );
}

#[test]
fn check_warns_about_silent_pitfalls_and_exits_0() {
    for (file, expected) in [
        (
            "no-cola.md",
            "tests/data/literate/no-cola.md: warning: no cola blocks in this file\n",
        ),
        (
            "unterminated.md",
            "tests/data/literate/unterminated.md:3:1: warning: unclosed cola fence runs to the end of the file\n",
        ),
        (
            "empty-entity.md",
            "tests/data/literate/empty-entity.md:5:5: warning: empty entity `llm/api`\n",
        ),
    ] {
        let path = format!("tests/data/literate/{file}");
        let out = colap(&["check", &path]);
        assert_eq!(code(&out), 0, "{file}: {}", stderr(&out));
        assert_eq!(stdout(&out), expected, "{file}");
    }
}

#[test]
fn check_takes_several_files() {
    let out = colap(&[
        "check",
        "tests/data/test_genite.md",
        "tests/data/duplicates/field-twice.md",
        "tests/data/literate/no-cola.md",
    ]);
    assert_eq!(code(&out), 1);
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].contains("field-twice.md:6:5: error:"));
    assert!(lines[1].contains("no-cola.md: warning:"));
}

#[test]
fn check_without_files_is_a_usage_error() {
    let out = colap(&["check"]);
    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("Usage"), "{}", stderr(&out));
}

#[test]
fn check_a_missing_file_is_an_error() {
    let out = colap(&["check", "tests/data/does-not-exist.md"]);
    assert_eq!(code(&out), 1);
    assert!(
        stdout(&out).starts_with("tests/data/does-not-exist.md: error: "),
        "{}",
        stdout(&out)
    );
}

#[test]
fn check_format_cola_round_trips_through_colap() {
    let out = colap(&[
        "check",
        "--format",
        "cola",
        "tests/data/duplicates/several-errors.md",
        "tests/data/literate/no-cola.md",
    ]);
    assert_eq!(code(&out), 1);
    let text = stdout(&out);
    assert!(text.starts_with("```cola\n"), "{text}");
    let parsed = colap::parse_str(&text, None).unwrap_or_else(|e| panic!("{e}\n{text}"));
    let findings = parsed
        .model
        .find_entity_by_path("finding")
        .expect("findings");
    let node = parsed.model.get_node(findings).expect("node");
    let node = node.borrow();
    assert_eq!(node.children.len(), 4);
    let severity = |i: usize| {
        parsed
            .model
            .get_field_value(node.children[i], "severity")
            .map(|v| v.to_string())
    };
    assert_eq!(severity(0).as_deref(), Some("\"error\""));
    assert_eq!(severity(3).as_deref(), Some("\"warning\""));
    assert_eq!(
        parsed
            .model
            .get_field_value(node.children[0], "line")
            .map(|v| v.to_string())
            .as_deref(),
        Some("6")
    );
}

#[test]
fn generate_is_the_default_command_and_also_works_by_name() {
    let dir = scratch("generate");
    let dir_str = dir.to_str().expect("utf-8 temp dir");

    let out = colap(&["tests/data/test_minimal.md", "-o", dir_str, "-m", "module"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(dir.join("test-minimal-config.rs").is_file());

    let out = colap(&[
        "generate",
        "tests/data/test_minimal.md",
        "-o",
        dir_str,
        "-m",
        "module",
        "-n",
        "other",
    ]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(dir.join("other.rs").is_file());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn generate_reports_findings_on_stderr_and_exits_1_on_errors() {
    let dir = scratch("generate-errors");
    let out = colap(&[
        "tests/data/duplicates/field-twice.md",
        "-o",
        dir.to_str().expect("utf-8 temp dir"),
        "-m",
        "module",
    ]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("field-twice.md:6:5: error: duplicate field"));
    assert!(!Path::new(&dir).exists(), "nothing is written on errors");
}

#[test]
fn trace_is_opt_in() {
    // rustemo's trace exists in debug builds only.
    if !cfg!(debug_assertions) {
        return;
    }
    let out = colap(&["check", "--trace", "tests/data/test_minimal.md"]);
    assert_eq!(code(&out), 0);
    assert!(stderr(&out).contains("Parsing started"), "{}", stderr(&out));
}
