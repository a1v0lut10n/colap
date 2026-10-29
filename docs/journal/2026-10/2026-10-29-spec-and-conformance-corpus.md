---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0005: a normative spec and a conformance corpus

## Context

The README was the language's only description, and it had drifted from
the grammar: trailing commas were advertised and not parsed. COLA-0001
through COLA-0004 decided what cola is — CommonMark fences, a cola-only
grammar, keys and slots, the diagnostics format, the canonical layout.
One normative document and a corpus of files with stated diagnostics were
needed to hold the parser, the highlighter and the README to it.

## Details

- `docs/spec/cola.md` has ten sections: scope and conformance, documents
  and fences, lexical structure, an EBNF grammar, the model with the
  reopening rules, diagnostics, the canonical layout, generated code, the
  open decisions (value language, schemas) stated as open, and the corpus
  contract. It is itself a clean literate document — every example is
  valid cola and a test keeps `colap check` on it clean; invalid shapes
  sit in `text` fences.
- The corpus lives in `tests/conformance/valid/` and `invalid/`: the
  literate and duplicate fixtures moved there, plus integer overflow,
  keywords used as names, and the value forms. Expected diagnostics are
  `NAME.expected` sidecars with a `<path>` placeholder — sidecars rather
  than embedded blocks, because a fence appended to the unterminated-fence
  case would close that fence.
- `tests/conformance.rs` checks every file against its sidecar, checks
  the spec, and compares the highlighter's identifier, number and string
  tokens on every cola line with the parser's located terminals, gathered
  by walking the AST. That test confirmed rustemo's columns are byte
  offsets with an exclusive end. Two exclusions on both sides, by design:
  lines inside a multi-line string, and names spelled like keywords (the
  parser accepts them by position; the highlighter shows keywords).
- The README's comparison table no longer claims trailing commas, and its
  syntax sections link to the spec. The integer-overflow message reads
  `integer out of range`.

Already in place before this task and only verified here: the single
grammar source (COLA-0001) and fence-level highlighter parity.

**Meta-impact.** aivolution-meta's documentation schemas have no `spec`
document type and no `docs/spec/` location. The spec carries
`type: spec` frontmatter and `aivolution lint` reports it as an
unrecognized location (a warning). A schema for normative specifications
would close that.

77 tests pass.

## Links

- Plan: `docs/implementation/2026-10/2026-10-29-cola-0005-spec-and-conformance.md`
- Task closed: `docs/tasks/2026-10/2026-10-29-normative-spec-and-conformance-corpus.md`
- The spec: `docs/spec/cola.md`
- Previous entries: `2026-10-29-fence-scanner-replaces-markdown-grammar.md`,
  `2026-10-29-duplicate-keys-keys-once-slots-reopen.md`,
  `2026-10-29-colap-check-and-file-accurate-diagnostics.md`,
  `2026-10-29-canonical-layout-and-colap-fmt.md`
