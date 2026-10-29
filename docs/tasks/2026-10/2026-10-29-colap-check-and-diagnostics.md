---
date: 2026-10-29
type: task
status: done
affects:
  - README.md
tags: [cli, diagnostics, tooling]
---

# Add `colap check` and file-accurate diagnostics

## Objective

colap can validate literate files without generating code. It reports every error as
`path:line:col: message` and returns lint-style exit codes, so downstream linters
and CI can gate on it.

## Context

- **Validating means generating code.** The only CLI path today is parse, then generate a
  crate or module. To validate a downstream project's files I had to run
  `RUSTEMO_NOTRACE=1 colap FILE -o /tmp/out -m module` and read the exit code.
- **Error reporting is thin.** Only the first parse error is reported. It is shown as
  `<str>:[line,col]` rather than the file path, and model errors are plain `String`s.
- **`SourceLocation.file_path` is always empty.** `ModelBuilder` sets it to
  `PathBuf::new()`.
- **Debug trace is on by default.** rustemo prints its trace unless `RUSTEMO_NOTRACE=1`
  is set. This is already noted in `docs/incubation/2026-09-21-standalone-generated-parsers.md`.

## Deliverables

1. **Subcommands.** `colap check FILE...` parses and builds the model; it writes nothing.
   `colap generate` keeps today's behaviour and remains the default for compatibility.
2. **Exit codes:** 0 clean, 1 errors, 2 usage. These are the same as flint `docs lint`.
3. **Message format.** One line per finding, in `path:line:col: severity: message`, with
   an optional `--format=cola` that emits findings as a cola block. JSON is not used.
4. **All errors are reported.** That covers parse errors (with recovery at block
   boundaries, so one bad block does not hide the next) and model errors (duplicates,
   from `2026-10-29-define-duplicate-key-semantics.md`).
5. **Warnings for silent pitfalls:**
   - a file with no cola blocks;
   - a file with an unterminated fence;
   - an empty entity.
6. **Locations are filled in.** `SourceLocation.file_path` is populated, and lines and
   columns refer to the original file.
7. **Quiet by default.** The rustemo trace is off unless `--trace` is given.
8. **A typed error enum** replaces `Result<_, String>` in the model and builder, so
   library consumers get structured diagnostics too.

## Explicitly out of scope

- Schema validation of field names and types (`2026-10-29-schema-validation.md`). This
  task only provides the reporting channel it will use.

## Completion

Before setting `status: done`:
- README.md documents `check`, the exit codes and the message format;
- a journal entry links back to this task.
