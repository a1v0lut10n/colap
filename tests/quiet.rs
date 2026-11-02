// SPDX-License-Identifier: Apache-2.0
//! The parser is quiet in a consumer's debug build
//! (`docs/tasks/2026-11/2026-11-01-quiet-parser-in-debug-builds.md`).
//!
//! rustemo's parse trace is compiled in under `debug_assertions` and
//! printed when `RUSTEMO_TRACE` is set. Before rustemo 0.8 the switch
//! ran the other way, and a library caller's own test run drowned in
//! trace. This test binary is such a debug build: it re-executes
//! itself to run one parse in a child process and reads what the
//! child wrote to stderr.

use std::path::Path;
use std::process::{Command, Output};

/// The parse under observation. Ignored, so it runs only when the
/// tests below ask the harness for it by name.
#[test]
#[ignore]
fn probe_parse() {
    let report = colap::check_file(Path::new("tests/data/test_minimal.md"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
}

fn run_probe(trace: bool) -> Output {
    let exe = std::env::current_exe().expect("the test binary knows its path");
    let mut cmd = Command::new(exe);
    cmd.args(["--exact", "probe_parse", "--ignored", "--nocapture"])
        .env_remove("RUSTEMO_TRACE");
    if trace {
        cmd.env("RUSTEMO_TRACE", "1");
    }
    cmd.output().expect("the probe runs")
}

#[test]
fn a_library_caller_in_a_debug_build_hears_nothing() {
    let out = run_probe(false);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.is_empty(), "the parser wrote to stderr:\n{stderr}");
}

#[test]
fn the_trace_is_still_there_for_whoever_asks() {
    // The trace exists in debug builds only.
    if !cfg!(debug_assertions) {
        return;
    }
    let out = run_probe(true);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("Parsing started"), "{stderr}");
}
