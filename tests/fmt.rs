// SPDX-License-Identifier: Apache-2.0
//! The canonical layout and `colap fmt`: exact output for a messy
//! fixture, the round-trip and idempotence properties over every
//! fixture, prose preserved, in-place rewriting, `--check`.
use colap::format::{format_file, format_literate_str, format_str};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn colap(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_colap"))
        .args(args)
        .output()
        .expect("run colap")
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("colap-fmt-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read")
}

#[test]
fn messy_formats_to_the_expected_text() {
    let formatted = format_file("tests/data/fmt/messy.md").expect("format");
    assert_eq!(formatted.text, read("tests/data/fmt/messy.expected.md"));
    assert_eq!(
        formatted.changed,
        vec![5],
        "the one cola block, by fence line"
    );

    let again = format_file("tests/data/fmt/messy.expected.md").expect("format");
    assert!(again.is_canonical());
    assert_eq!(again.text, read("tests/data/fmt/messy.expected.md"));
}

/// Every fixture that is free of errors: parsing the canonical text
/// gives an equal model, and formatting twice equals formatting once.
#[test]
fn every_clean_fixture_round_trips_and_is_idempotent() {
    let mut checked = 0;
    for dir in [
        "tests/data",
        "tests/data/fmt",
        "tests/conformance/valid",
        "tests/conformance/invalid",
    ] {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "md" | "cola") {
                continue;
            }
            let Ok(parsed) = colap::parse_file(&path) else {
                continue;
            };
            // Model round trip.
            let canonical = parsed.model.to_cola();
            let reparsed = colap::parse_cola_str(&canonical, None)
                .unwrap_or_else(|e| panic!("{}: {e}\n{canonical}", path.display()));
            assert_eq!(
                reparsed.model.to_string(),
                parsed.model.to_string(),
                "{}",
                path.display()
            );
            assert_eq!(reparsed.model.to_cola(), canonical, "{}", path.display());
            // File round trip.
            let once = format_file(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let twice = if ext == "md" {
                format_literate_str(&once.text, Some(&path))
            } else {
                format_str(&once.text, Some(&path))
            }
            .expect("format twice");
            assert!(twice.is_canonical(), "{}", path.display());
            assert_eq!(twice.text, once.text, "{}", path.display());
            checked += 1;
        }
    }
    assert!(checked >= 20, "only {checked} fixtures checked");
}

#[test]
fn a_bare_file_is_formatted_whole() {
    let formatted = format_file("tests/conformance/valid/bare.cola").expect("format");
    assert_eq!(formatted.text, "root_entity: name: \"bare\";\n");
    assert_eq!(formatted.changed, vec![1]);
}

#[test]
fn fmt_rewrites_in_place_and_check_reports_the_block() {
    let dir = scratch("in-place");
    let file = dir.join("messy.md");
    std::fs::copy("tests/data/fmt/messy.md", &file).expect("copy");
    let file_str = file.to_str().expect("utf-8 path");

    let out = colap(&["fmt", "--check", file_str]);
    assert_eq!(code(&out), 1);
    assert_eq!(
        stdout(&out),
        format!("{file_str}:5:1: error: not canonical; run `colap fmt`\n")
    );
    assert_eq!(
        read(&file),
        read("tests/data/fmt/messy.md"),
        "--check writes nothing"
    );

    let out = colap(&["fmt", file_str]);
    assert_eq!(code(&out), 0, "{}", stdout(&out));
    assert_eq!(read(&file), read("tests/data/fmt/messy.expected.md"));

    let out = colap(&["fmt", "--check", file_str]);
    assert_eq!(code(&out), 0);
    assert_eq!(stdout(&out), "");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fmt_refuses_a_file_with_errors_and_leaves_it_alone() {
    let dir = scratch("errors");
    let file = dir.join("field-twice.md");
    std::fs::copy("tests/conformance/invalid/field-twice.md", &file).expect("copy");
    let file_str = file.to_str().expect("utf-8 path");

    let out = colap(&["fmt", file_str]);
    assert_eq!(code(&out), 1);
    assert!(
        stdout(&out).contains(":6:5: error: duplicate field `x` in `a`"),
        "{}",
        stdout(&out)
    );
    assert_eq!(
        read(&file),
        read("tests/conformance/invalid/field-twice.md")
    );

    let _ = std::fs::remove_dir_all(&dir);
}
