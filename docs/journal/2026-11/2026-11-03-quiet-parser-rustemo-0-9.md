---
date: 2026-11-03
type: journal
components: []
aspects: []
tags: [library, consumer, rustemo, diagnostics]
---

# A quiet parser: rustemo 0.9.1 (COLA-0014)

## Context

The first library consumer of colap 0.4, a linter over a literate cola
knowledge repository, found its test runs drowning in rustemo's parse
trace: about 3.3 million lines of stderr for one lint. The task of
2026-11-01 asked for a reproduction, the root cause, and a fix that
keeps the trace opt-in without touching the process environment from
inside the library.

## Details

The root cause is in rustemo 0.7.1, not in colap. Its `log!` macros
are compiled in under `debug_assertions` and print unless
`RUSTEMO_NOTRACE` is set, reading the variable on every call; they
expand inside rustemo's own parser and lexer as well as in the generated
`cola.rs`, so the consumer's profile governs the trace. The compiler's
`notrace(true)` that `build.rs` asked for only set the variable inside
the build script's own process.

rustemo 0.8.0 turned the switch around: the trace is off unless
`RUSTEMO_TRACE` is set. That is the task's second preference, an
upstream option, so COLA-0014 bumps rustemo and rustemo-compiler to
0.9.1. claimr and phenotyper already pin it, and aicogito's lockfile
carried both 0.7.1 and 0.9.1, so the bump unifies the siblings.

The port is mechanical: `Location` became `SourceSpan` with a mandatory
end and `Option` line and column, the builder's `location` field became
`span`, `ValLoc` became `ValSpan`, and the parse error is
`Error::ParseError(Box<ParseError>)` whose `message` is the verdict
alone, the excerpt now living on `Display`. The conformance corpus
holds every line and column unchanged. `colap --trace` sets
`RUSTEMO_TRACE`; without the flag the environment is left as the caller
had it. `tests/quiet.rs` is the reproduction the task asked for: the
test binary, itself a debug build, re-executes itself to parse one file
in a child process and reads an empty stderr, and reads the trace when
it asks for it.

Breaking for callers of the raw AST through `colap::rustemo`, so this
ships in 0.5.0 together with the feature gates of the companion task.

## Links

- Task: `docs/tasks/2026-11/2026-11-01-quiet-parser-in-debug-builds.md`
- Plan: `docs/implementation/2026-11/2026-11-03-cola-0014-quiet-parser.md`
- Planning entry: `docs/journal/2026-11/2026-11-01-plan-library-consumer-fixes.md`
- rustemo 0.8.0 changelog: "Debug trace is now off by default" (PR #23)
