---
date: 2026-10-30
type: task
ticket: COLA-0007
status: done
affects:
  - src/grammar/cola.rustemo
  - src/model/config_model.rs
  - src/model/model_builder.rs
  - src/model/canonical.rs
  - src/highlight.rs
  - src/generator/generator_impl.rs
  - docs/spec/cola.md
  - README.md
tasks:
  - docs/tasks/2026-10/2026-10-29-value-language-extensions.md
---

# COLA-0007: the value forms of the value-language design

## Premise

`docs/design/2026-10-30-value-language.md` decides the value language.
This ticket implements the forms that fall out of the grammar: symbols,
lists, references, decoded escapes and block strings, number
separators and exponents, trailing commas. Comments, which need the
model to carry them through the formatter, follow in their own ticket.

## Action items

1. **Grammar**: `Value` gains `Symbol` (an identifier), `List`,
   `Reference`, `BlockString`; `Number` gains separators and
   exponents; a field list may end in `,` or in `, Entity`; a list may
   end in `,`.
2. **Model**: `ConfigValue::{Symbol, List, Reference}`; the builder
   decodes escapes and block strings and rejects unknown escapes;
   `finish` resolves every reference and reports the unresolved ones.
3. **Canonical layout**: the forms of the design record, including the
   multi-line list, the block string and the long-float rule.
4. **Highlighter**: symbol and reference tokens, block strings across
   lines, the new number form.
5. **Generator**: `String` for symbols and references, `Vec<T>` for
   lists, with the template reading each variant.
6. **Spec, README, corpus**: sections 3, 4, 5.5, 7 and 9; the
   comparison table; new valid files for every form and invalid files
   for an unknown escape, a nested list, an unresolved reference; the
   existing escape fixture updated per the migration note.
7. **Migration note** in the journal entry: escape decoding is the one
   change of meaning.

## What was built

As planned, with three things learned on the way:

- The grammar's definition rule is `Item*` with `Item: Field | Field
  Comma | Entity`: the comma belongs to the field before it, the
  repetition is a vector rather than a left-recursive chain (a block of
  thousands of entries builds no deep AST), and there is no conflict. A
  comma never follows an entity. Two fields with no comma between them
  were already accepted by the 0.2 grammar; the spec now says so and the
  canonical layout always writes the comma.
- `colap fmt` formats one block at a time, so a reference into another
  block cannot resolve there; `ModelBuilder::finish_block` finishes a
  model without resolving references, for a document that was already
  checked whole.
- The generator's templates were HTML-escaped by handlebars, which only
  showed once a type contained `<`; escaping is now off, and the entity
  template resolves its import path from the template root inside
  `each` blocks, where it used to render empty.

## Verification

- [x] The three Rust gates are clean (clippy covers the bench).
- [x] Every corpus file's diagnostics equal its sidecar (3 new valid
      files, 3 new invalid ones); the spec is clean; token parity holds
      for symbols, references and block strings.
- [x] `tests/values.rs` pins every decoded value form, the unresolved
      reference error and the trailing-comma equivalences.
- [x] A crate generated from a sample with lists, symbols and references
      compiles against this checkout (its legacy integration test still
      names `Llms`, as before).
- [x] `cargo bench --bench scale` still passes (1.02x).
