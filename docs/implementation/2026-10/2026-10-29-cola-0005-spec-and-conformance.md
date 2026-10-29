---
date: 2026-10-29
type: task
ticket: COLA-0005
status: done
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

- [x] The three Rust gates are clean (77 tests; 4 new in
      `tests/conformance.rs`).
- [x] `colap check docs/spec/cola.md` is clean, and a test keeps it so.
- [x] Every corpus file's diagnostics equal its sidecar (25 files, 14
      sidecars).
- [x] `aivolution lint .` has no errors; README updated; journal entry
      proposed.

## Outcome

Built as planned. Notes beyond the action items:

- The token-level parity test walks the AST for located terminals
  (identifiers, numbers, strings) and compares them with the highlighter's
  tokens on every cola line of every corpus file, the fixtures and the
  spec. It confirmed that rustemo's columns are byte offsets with an
  exclusive end. Two things are left out on both sides by design: lines
  inside a multi-line string (a line lexer cannot see one token across
  lines) and names spelled like keywords (the highlighter shows them as
  keywords; the spec says so).
- Corpus files added beyond the moved fixtures: `integer-overflow.md`,
  `keywords-as-names.md`, `values.md`.
- The linter has no document type or location for a spec:
  `docs/spec/cola.md` carries `type: spec` frontmatter and is reported as
  an unrecognized location (a warning, not an error). That is a gap for
  aivolution-meta's schemas, recorded in the journal entry.
- Already done before this task and only verified here: the single
  grammar source (COLA-0001) and the fence-level highlighter parity.
