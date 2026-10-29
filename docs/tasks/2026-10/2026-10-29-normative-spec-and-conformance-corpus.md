---
date: 2026-10-29
type: task
status: done
affects:
  - README.md
tags: [spec, testing, housekeeping]
---

# Write a normative cola spec with a conformance corpus, and remove grammar drift risks

## Objective

The language has one normative specification and one grammar source. A conformance
corpus of valid and invalid files checks the parser, the highlighter and the README
against each other.

## Context

- **Advertised and actual syntax disagree.** The README is today's only description of
  the language, and it differs from the grammar:
  - trailing commas are advertised but not parsed;
  - "first-class markdown embedding" breaks on any foreign fence
    (`2026-10-29-parse-non-cola-fenced-blocks.md`).
- **Tests are positive only.** `tests/data/` holds positive examples with no expected
  failures.
- **Two grammar copies.** `build.rs` copies `src/grammar/cola.rustemo` to
  `src/cola.rustemo` only when the destination is missing, so an edit to the canonical
  grammar can silently not take effect while the stale copy remains. Both copies are
  identical today.
- **The highlighter is a separate lexer.** `src/highlight.rs` is a hand-written line
  lexer that can drift from the grammar.

## Deliverables

1. **The spec.** `docs/spec/cola.md`, itself literate, defines:
   - lexical rules and the markdown embedding rules;
   - the entity, plural and field grammar;
   - value forms;
   - merge and duplicate semantics, including the outcome of each pending ADR.
2. **The corpus.** `tests/conformance/valid/*.md` and `tests/conformance/invalid/*.md`,
   each invalid file stating the expected `path:line:col` diagnostic. The corpus is
   seeded with every reproduction from the 2026-10-29 tasks.
3. **Single grammar source.** The build always regenerates from `src/grammar/cola.rustemo`
   and stops keeping a second copy, or fails if the two differ.
4. **Highlighter parity.** A test asserts that the token classes the highlighter assigns
   on the corpus agree with the parser's tokens.
5. **README sync.** README.md's syntax claims link to the spec and are corrected where
   they are wrong.

## Explicitly out of scope

- New language features. The spec records what exists plus whatever the ADRs accept.

## Completion

Before setting `status: done`:
- README.md matches the spec;
- a journal entry links back to this task.
