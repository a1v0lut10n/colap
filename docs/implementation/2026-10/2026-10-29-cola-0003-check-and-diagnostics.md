---
date: 2026-10-29
type: task
ticket: COLA-0003
status: in-progress
affects:
  - README.md
  - src/main.rs
tasks:
  - docs/tasks/2026-10/2026-10-29-colap-check-and-diagnostics.md
---

# COLA-0003: `colap check` and file-accurate diagnostics

## Premise

COLA-0001 and COLA-0002 built the error surface: file-accurate parse
errors, duplicates with both locations, every error of a file collected in
line order, recovery at block boundaries. What is missing is the way to
*use* it without generating code: a `check` subcommand, lint-style exit
codes, a `severity`-tagged message format, warnings for the pitfalls that
are silent today (no cola blocks, an unterminated fence, an empty entity),
and a quiet default — rustemo's debug-build trace prints unless
`RUSTEMO_NOTRACE` is set, and it checks that variable at run time, so the
CLI can decide.

Warnings must survive errors: an unterminated fence is usually the cause
of the parse error that follows it, so the reading step has to return both.

## Action items

1. **`Report`** in `src/lib.rs`: `check_file` / `check_str` /
   `check_literate_str` / `check_cola_str` never fail; they return the
   outcome (model or error) *and* the warnings. The `parse_*` functions
   become wrappers that drop the warnings on error.
2. **Warnings** (`src/diagnostic.rs`): `Warning::Fence` (the scanner's),
   `NoColaBlocks` for a literate file without a cola fence, `EmptyEntity`
   for an entity with neither fields nor children, each with a location
   where one exists.
3. **Diagnostics** (`src/diagnostic.rs`): one `Diagnostic` shape
   (severity, path, line, column, message) built from errors and
   warnings; `Display` is `path:line:col: severity: message`; `to_cola`
   renders a list of findings as a cola block.
4. **CLI** (`src/main.rs`): `colap check FILE...` with `--format text|cola`;
   `colap generate` keeps today's behaviour and stays the default when no
   subcommand is given; `--trace` on every command; exit 0 clean, 1 errors,
   2 usage (clap's own). The trace is off unless `--trace`.
5. **Typed errors everywhere**: `ModelBuilder::build_config_model` returns
   `Error`, not `String`.
6. **Tests** (`tests/cli.rs`): the binary on the existing fixtures — exit
   codes, exact message lines, warnings, `--format cola` output that colap
   itself parses, generate by default and by name, trace off by default
   and on with `--trace`.
7. **README**: the subcommands, exit codes, message format and the trace
   flag.

## Verification

- [ ] The three Rust gates are clean.
- [ ] `colap check` on every fixture under `tests/data/` gives the
      expected exit code and lines.
- [ ] `colap check --format cola` output round-trips through `parse_str`.
- [ ] `aivolution lint .` clean; README updated; journal entry proposed.

## Outcome

(filled in before the change proposal)
