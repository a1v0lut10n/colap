---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0003: `colap check` and file-accurate diagnostics

## Context

Validating a literate file meant generating code into a scratch directory
and reading the exit code. Only the first error was shown, as `<str>:[line,col]`
rather than a path, and rustemo's debug trace printed unless
`RUSTEMO_NOTRACE=1` was set. COLA-0001 and COLA-0002 built the error
surface — file-accurate locations, every error of a file collected,
recovery at block boundaries. COLA-0003 adds the way to use it.

## Details

- `colap check FILE...` validates and writes nothing. Each finding is one
  line, `path:line:col: severity: message`, errors and warnings sorted
  together by line. `--format cola` renders the findings as a
  `finding plural findings` block that colap itself parses. Exit codes
  are lint-style: 0 clean, 1 errors, 2 usage.
- `colap generate` keeps today's behaviour and remains the default when
  no subcommand is given; it now prints the same diagnostics to stderr
  and exits 1 on errors.
- Three warnings for pitfalls that used to be silent: a literate file
  with no cola blocks, an unclosed fence running to the end of the file,
  and an empty entity, with its location.
- Warnings survive errors. The new `check_*` functions return a `Report`
  carrying the outcome and the warnings, so an unclosed fence can explain
  the parse error that follows it. The `parse_*` functions are wrappers
  over them, and `build_config_model` returns the typed error.
- The trace finding: rustemo's trace exists in debug builds only and
  checks `RUSTEMO_NOTRACE` at every call, so the CLI sets or clears the
  variable in `main` before any thread exists — one `unsafe` block with
  its invariant stated. Consumers' own debug builds still need the
  variable; release builds are silent.

Ten CLI tests run the built binary against the fixtures with exact
output; 65 tests pass. Schema validation keeps its own task and reports
through this channel.

## Links

- Plan: `docs/implementation/2026-10/2026-10-29-cola-0003-check-and-diagnostics.md`
- Task closed: `docs/tasks/2026-10/2026-10-29-colap-check-and-diagnostics.md`
- Previous entries: `2026-10-29-fence-scanner-replaces-markdown-grammar.md`,
  `2026-10-29-duplicate-keys-keys-once-slots-reopen.md`
