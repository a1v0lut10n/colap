---
date: 2026-10-29
type: task
ticket: COLA-0004
status: done
affects:
  - README.md
  - src/model/config_model.rs
tasks:
  - docs/tasks/2026-10/2026-10-29-ordered-model-and-canonical-formatter.md
---

# COLA-0004: source order everywhere, and `colap fmt`

## Premise

Downstream tools regenerate literate files and treat any hand edit as
drift, which only works if writing cola is byte-deterministic. Since
COLA-0002 the model keeps fields and children in source order, so half of
the first deliverable is done; the generated plural wrappers still hand
out `HashMap` order, and nothing can write cola back out.

A canonical layout has to be a pure function of the model, so that
formatting twice equals formatting once, and it must round-trip: parsing
the output gives the same model. Two details decide that: floats must
keep their decimal point (`8.0`, never `8`, or the value turns into an
integer), and a string's quotes must be chosen so the raw text between
them is unchanged (escapes are not decoded yet).

## Action items

1. **Generated plural wrappers** keep entry order: a `Vec<(String, T)>`
   with a name index beside it (no new dependency — the standard library
   does it). `get`, `keys`, `values`, `iter`, `insert`, `count` as before,
   now ordered.
2. **The writer** `ConfigModel::to_cola()` (`src/model/canonical.rs`):
   four-space indentation; an entity with no children whose one-line form
   fits in 80 columns is written on one line, otherwise one field per
   line with the closing `;` on its own line; fields before children; no
   trailing commas; floats always carry a decimal point; strings are
   double-quoted unless their raw text contains a double quote, then
   single-quoted. `ConfigValue`'s `Display` follows the same float rule.
3. **`colap fmt FILE...`** (`src/format.rs` plus the subcommand): runs the
   full check first and refuses files with errors (findings printed, exit
   1); then rewrites only the bodies of cola fences, each block formatted
   on its own, prose untouched byte for byte; bare `.cola` files are
   formatted whole. `--check` writes nothing and exits 1 when a block is
   not canonical, naming the block's fence line.
4. **Tests** (`tests/fmt.rs`): the round-trip property over every
   fixture that parses (parse → format → parse gives an equal model;
   format twice equals once); exact canonical output for a messy fixture;
   prose preserved; `--check` before and after; bare files.
5. **README**: `fmt`, the canonical layout, the order guarantee.

## Verification

- [x] The three Rust gates are clean (73 tests; 5 new in `tests/fmt.rs`,
      3 in `canonical.rs`).
- [x] Every clean fixture round-trips (parse → format → parse gives an
      equal model) and is idempotent under `fmt`.
- [x] `colap fmt --check` on a freshly formatted file exits 0; on the messy
      fixture it names the block's fence line and exits 1.
- [x] `aivolution lint .` clean; README updated; journal entry proposed.

## Outcome

Built as planned. Notes beyond the action items:

- `ConfigValue`'s `Display` now prints floats with a decimal point too,
  so the model's own text rendering agrees with the writer and the
  round-trip test can compare models by it.
- `format_literate_str` runs the whole-file check first, then formats
  each block through its own one-block model; a block body at the end of
  an unterminated fence gains its final newline.
- The width rule counts characters, not bytes, so non-ASCII strings do
  not change the layout decision.
- The round-trip test chooses the literate path for `.md` files
  explicitly: dialect detection on a markdown file with no cola fence is
  bare cola by design.
