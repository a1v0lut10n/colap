---
date: 2026-10-29
type: task
ticket: COLA-0005
status: in-progress
affects:
  - README.md
  - docs/spec/cola.md
tasks:
  - docs/tasks/2026-10/2026-10-29-normative-spec-and-conformance-corpus.md
---

# COLA-0005: a normative spec and a conformance corpus

## Premise

The README has been the language's only description, and it has drifted
from the grammar (trailing commas are advertised and not parsed). Four
PRs have since decided what the language is: CommonMark fences, a cola-only
grammar, the duplicate rules, the diagnostics format, the canonical
layout. One document should state all of that, and a corpus of files with
stated diagnostics should hold the parser, the highlighter and the README
to it.

Two of the task's deliverables are already done: the build processes the
single grammar in `src/grammar/` (COLA-0001), and the highlighter reuses
the scanner's fence rules with a line-level parity test. What remains is
the token level.

## Action items

1. **The spec** `docs/spec/cola.md`, itself a clean literate document:
   documents and fences, lexical rules, the grammar, values, the model
   with keys and slots and the reopening rules, diagnostics, the
   canonical layout, conformance, and the open decisions (value language,
   schemas) stated as open.
2. **The corpus** `tests/conformance/valid/` and `tests/conformance/invalid/`:
   the literate and duplicate fixtures move there, seeded with every
   reproduction from the 2026-10-29 tasks, plus integer overflow. Each
   file may have a `NAME.expected` sidecar listing its diagnostics with a
   `<path>` placeholder; a sidecar rather than an embedded block, because
   an appended fence would close the unterminated-fence case.
3. **The runner** `tests/conformance.rs`: every valid file has no errors
   and exactly the stated warnings; every invalid file has errors and
   exactly the stated diagnostics; the spec checks clean; and the
   highlighter's identifier, number and string tokens on every cola line
   are exactly the parser's located terminals (token-level parity).
4. **Existing tests** keep passing against the moved fixtures.
5. **README sync**: the comparison table stops claiming trailing commas,
   and the syntax sections link to the spec.
6. **Message polish**: the integer-overflow error reads
   `integer out of range: N`.

## Verification

- [ ] The three Rust gates are clean.
- [ ] `colap check docs/spec/cola.md` is clean.
- [ ] Every corpus file's diagnostics equal its sidecar.
- [ ] `aivolution lint .` clean; README updated; journal entry proposed.

## Outcome

(filled in before the change proposal)
