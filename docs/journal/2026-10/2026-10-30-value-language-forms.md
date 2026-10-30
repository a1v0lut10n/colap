---
date: 2026-10-30
type: journal
components: []
aspects: []
---

# COLA-0007: the value language, decided and its forms implemented

## Context

The field test's value-language task asked for one design record and
then the forms. The record, `docs/design/2026-10-30-value-language.md`,
accepts bare symbols, bracketed lists of scalars, `@path` references
that `colap check` resolves, `//` comments carried through `fmt`,
decoded escapes with `"""` block strings, digit separators and
exponents, and trailing commas. Dates, nulls and decimal-as-text are
deferred to schemas. The one point debated was the trailing comma: the
canonical multi-line form keeps it, so that appending a field or an
entry never touches the line before it, while the one-line form stays
`name: a: 1, b: "x";`.

## Details

COLA-0007 implements every form but comments, which need the model to
carry them and follow in their own ticket. The grammar's definition
rule became `Item*` with `Item: Field | Field Comma | Entity`: the comma
belongs to the field before it, there is no LR(1) conflict, and the
repetition is a vector, so a block of thousands of entries builds no
deep AST. Lists are `Scalar*[Comma]`; a comma never follows an entity.

Migration note: escape decoding is the one change of meaning for
existing files. `"say \"hi\""` was fourteen characters of raw text and
is now `say "hi"`; a backslash that is not one of `\" \' \\ \n \r \t
\u{…}` is now an error. The downstream linter forbade backslashes, so
the knowledge repository is unaffected; `tests/conformance/valid/
values.md` was the one fixture that changed meaning.

Found on the way: two fields with no comma between them were already
accepted by the 0.2 grammar, which the spec now states; `fmt` formats
one block at a time, so references into other blocks are resolved only
by the whole-file check that precedes formatting; and the generator's
templates were HTML-escaped by handlebars, visible only once a type
contained `<`.

## Links

- Design: `docs/design/2026-10-30-value-language.md`
- Plan: `docs/implementation/2026-10/2026-10-30-cola-0007-value-language.md`
- Task: `docs/tasks/2026-10/2026-10-29-value-language-extensions.md`
- Spec sections 3, 4, 5.5, 6, 7 and 9 in `docs/spec/cola.md`
