---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0004: a canonical layout and `colap fmt`

## Context

Downstream tools regenerate literate files and treat any hand edit as
drift, which only works if writing cola is byte-deterministic. COLA-0002
had already put fields and children in source order; nothing could write
cola back out, and the generated plural wrappers handed out `HashMap`
order.

## Details

- `ConfigModel::to_cola()` (`src/model/canonical.rs`) is a pure function
  of the model: four spaces per level; an entity with no children whose
  one-line form fits in 80 columns is written on one line, otherwise one
  field per line with the closing `;` on its own line; fields before
  children, in source order, no trailing comma. Floats always carry a
  decimal point so `8.0` stays a float, and strings are quoted so the raw
  text between the quotes is unchanged — the two details that make the
  round trip exact while escapes are still undecoded.
- `colap fmt FILE...` (`src/format.rs`) checks the whole file first and
  refuses files with errors; then it rewrites only the bodies of cola
  fences, each block through its own one-block model, so prose, foreign
  fences and a collection split by narrative stay exactly as they were.
  `--check` writes nothing and names each non-canonical block by its
  fence line.
- Generated plural wrappers keep entry order with a `Vec` and a name index
  beside it; `keys`, `values` and the new `iter` walk in file order. No
  new dependency.
- `ConfigValue`'s `Display` follows the writer's float rule, so the
  round-trip test can compare models by their text rendering.

The property test runs over every clean fixture in the tree: parse, then
format, then parse gives an equal model, and formatting twice equals
formatting once. 73 tests pass.

## Links

- Plan: `docs/implementation/2026-10/2026-10-29-cola-0004-ordered-model-and-fmt.md`
- Task closed: `docs/tasks/2026-10/2026-10-29-ordered-model-and-canonical-formatter.md`
- Previous entries: `2026-10-29-fence-scanner-replaces-markdown-grammar.md`,
  `2026-10-29-duplicate-keys-keys-once-slots-reopen.md`,
  `2026-10-29-colap-check-and-file-accurate-diagnostics.md`
