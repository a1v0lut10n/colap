---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0001: a fence scanner replaces the markdown grammar

## Context

Two field-test defects recorded on 2026-10-29 shared one root cause: the
LR grammar modelled the whole markdown file, so a foreign fence never
closed (the closing line tied between a code line and a fence end) and a
prose line starting with a backtick or a bare `#` matched no terminal.
COLA-0001 took markdown out of the grammar.

## Details

- `src/literate.rs` is a CommonMark-style fence scanner: backtick or tilde
  runs of three or more, up to three spaces of indentation, any info string
  with the language as its first word, a closer of the same character at
  least as long. Bodies are kept verbatim with their file line offset; an
  unterminated fence is a warning naming its opening line.
- The grammar's root is now `Entity*`; every markdown rule is gone.
- `ModelBuilder` is incremental (`new`, `add_block`, `finish`) and maps
  every `SourceLocation` back to the original file, path included.
- A typed `Error` (`src/error.rs`) prints `path:line:col: message` with
  rustemo's verdict line only.
- A facade — `parse_file`, `parse_str`, `parse_literate_str`,
  `parse_cola_str` — replaces direct `ColaParser` use, and generated
  crates no longer depend on rustemo. `rustemo` is re-exported for callers
  that want the raw AST.
- `src/highlight.rs` reuses the scanner's fence functions; `LineState`
  carries the opening fence, and a test asserts scanner and highlighter
  agree on every fixture.
- `build.rs` processes the one grammar in `src/grammar/` directly; the
  duplicate `src/cola.rustemo` and the stale generated copies beside the
  grammar are gone.

Breaking for 0.3.0: `ColaParser::parse` takes one cola block, not a
document; `LineState::Cola` and `Code` carry a `Fence`; `ModelBuilder` is
a struct. Downstream: chaiss's `cola_config.rs` and aicogito's
`syntax.rs` each need a one-line change when they bump.

39 tests pass, 12 of them new in `tests/literate.rs` over the fixtures in
`tests/data/literate/`, seeded with every reproduction from the two tasks.

## Links

- Plan: `docs/implementation/2026-10/2026-10-29-cola-0001-fence-scanner.md`
- Tasks closed: `docs/tasks/2026-10/2026-10-29-parse-non-cola-fenced-blocks.md`,
  `docs/tasks/2026-10/2026-10-29-accept-any-prose-line.md`
- Incubation served: `docs/incubation/2026-09-21-standalone-generated-parsers.md`
- Planning entry: `2026-10-29-plan-literate-format-hardening.md`
