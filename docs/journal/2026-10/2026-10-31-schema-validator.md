---
date: 2026-10-31
type: journal
components: []
aspects: []
tags: [schema, validation, time]
---

# COLA-0011: the schema validator, `colap check --schema`

## Context

The schema dialect was accepted on 2026-10-31 and exact numbers
(COLA-0010) landed first so that bounds compare exactly. This ticket
builds the validator, its meta-schema and the schema spec, and takes
the schema task's deliverables 1, 2 and 4; the generator's `--schema`
is the one left.

## Details

Planning added `like` to the dialect: the meta-schema has to declare
a declaration that contains declarations, which the mirrored tree
can only say by naming one. `like: @path` is a cola reference to
another declaration, resolved by the parser, bounded by the
document's depth; it also serves any shape that repeats.

A schema loads in three steps: read as any document, validated
against the meta-schema (`src/schema/meta.md`, shipped in the crate
and validating itself), then built with the checks no meta-schema can
state: `one_of` literals of the field's type, `min` not above `max`,
patterns compiling, constraints fitting their types, `like` pointing
at a declaration, the `_utc` naming rule. Findings are `Diagnostic`s
on `Report::findings`, counted by `is_clean` and merged into
`diagnostics()`, so editors and linters see them beside parse errors.
`colap check --schema` prints a broken schema's findings alone and
validates nothing against it.

Temporal types are parsed by hand except instants, which `chrono`
reads: `date` at three precisions, `datetime` civil with an optional
bracketed IANA zone, `edtf` levels 0 and 1 with the qualifier after
the date as the specification writes it. The spec tables the forms.

The corpus gained a `schema` directory of schema and document pairs
with sidecars, including a `like`-recursive tree and three schemas
that must fail to load; the runner checks every pair and both specs.

## Links

- Plan: `docs/implementation/2026-10/2026-10-31-cola-0011-schema-validator.md`
- Design: `docs/design/2026-10-31-schema-dialect.md`
- Spec: `docs/spec/schema.md`; `docs/spec/cola.md` sections 8, 9, 10
- Task: `docs/tasks/2026-10/2026-10-29-schema-validation.md`
