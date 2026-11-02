---
date: 2026-11-03
type: task
ticket: COLA-0014
status: done
affects:
  - Cargo.toml
  - build.rs
  - src/error.rs
  - src/model/model_builder.rs
  - src/main.rs
  - tests/cli.rs
  - tests/quiet.rs
  - tests/conformance.rs
  - README.md
  - CHANGELOG.md
  - docs/tasks/2026-11/2026-11-01-quiet-parser-in-debug-builds.md
---

# COLA-0014: a quiet parser in a consumer's debug build

Executes `docs/tasks/2026-11/2026-11-01-quiet-parser-in-debug-builds.md`:
a crate that depends on colap and builds in its dev profile gets no
parse trace on stderr unless it asks for one.

## Root cause

rustemo 0.7.1's `log!`/`logn!` macros are compiled in under
`debug_assertions` and print unless the `RUSTEMO_NOTRACE` environment
variable is set, reading it on every call. They expand inside rustemo's
own LR parser and lexer as well as in the generated `src/parser/cola.rs`,
so the trace is governed by the consumer's profile, not by colap. The
compiler setting `notrace(true)` in `build.rs` only set that variable
inside the build-script process, which reaches nothing at run time.

rustemo 0.8.0 turned the switch around ("debug trace is now off by
default"): the macros print only when `RUSTEMO_TRACE` is set. claimr and
phenotyper already pin 0.9.1, and aicogito's lockfile carried both
0.7.1 (through colap) and 0.9.1 (through claimr).

## Decision

Take the upstream fix: rustemo and rustemo-compiler 0.9.1, the task's
second preference, since the first (a parser setting) does not exist and
the environment variable is ruled out. No colap-side switch is added;
rustemo's own `RUSTEMO_TRACE` is the one way to ask for the trace, and
`colap --trace` sets it.

## Action items

- **A1** — `rustemo = "0.9.1"`, `rustemo-compiler = "0.9.1"`; drop
  `notrace(true)` from `build.rs` (the setting is gone).
- **A2** — Port to 0.9.1's types: `rustemo::Error::ParseError(Box<ParseError>)`
  with `message` and `span` in `src/error.rs`; `SourceSpan`/`Position`
  (line and column now `Option<usize>`, end no longer optional) and the
  builder's `span` field in `src/model/model_builder.rs`; `ValSpan` in
  `tests/conformance.rs`.
- **A3** — `colap --trace` sets `RUSTEMO_TRACE`; without the flag the
  environment is left alone, so an exported variable still works.
- **A4** — Reproduction test `tests/quiet.rs`: the test binary, a debug
  build, re-executes itself to run one `check_file` in a child process
  with `RUSTEMO_TRACE` unset and asserts an empty stderr; a second test
  sets it and asserts the trace is there.
- **A5** — README says how a library caller gets the trace; CHANGELOG
  notes the bump under Unreleased, with the re-export change as
  breaking for callers of the raw AST.

## What was built

A1 to A5 as written. The port is mechanical and the conformance corpus
holds every line and column unchanged, so rustemo's coordinates did not
move between 0.7 and 0.9. The parse error message is now the verdict
alone (`parse.message`); rustemo's excerpt with the span marked moved
to the error's `Display`, so the last-line stripping is gone. The
lockfile swaps `colored` and `lazy_static` for `yansi`, `codesnake`,
`is-terminal` and `unicode-width`.

## Verification

- [x] `tests/quiet.rs` passes in a debug build.
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test`.
- [x] `colap check --trace` still prints the trace (`tests/cli.rs::trace_is_opt_in`).
