---
date: 2026-10-31
type: journal
components: []
aspects: []
tags: [schema, generator]
---

# COLA-0012: `colap generate --schema`, types from declarations

## Context

The schema task's last deliverable: with a schema, the generator
types from declarations instead of guessing from the one literal a
sample holds. This closes the task opened on 2026-10-29.

## Details

A second generator rather than a patch on the first. The inference
path walks a sample's model and keeps its oddities; the schema path
walks the schema's shapes: one struct per shape named after its
declaration, parent-prefixed on collision, a collection wrapper per
plural declaration, enums for `symbol` fields with `one_of`, `Option`
for what is not required, `Vec` for lists, colap's exact `Number` for
`decimal` (the record said `String`, written before the type
existed), `Option<ConfigValue>` for `any`. A `like` declaration has
its target's type, so a tree is `Folder` holding `Folders` of
`Folder`. Readers are closures the generator writes and the template
places, one arm per field shape rather than per type.

The schema travels with the crate: the generated code embeds its
text and `Root::load(path)` reads, validates and builds, or returns
every error. The CLI takes the input optionally with a schema,
validates it first and loads it from the crate's integration test.
`--colap-path` writes a path dependency for local builds; without
it the generated crate depends on the published colap, replacing the
`../colap` relative path that only worked beside a checkout.

The test builds a crate generated from the registry schema against
the checkout and runs it: typed values, enums, `Number`, and `load`
rejecting the findings document with the validator's errors.

## Links

- Plan: `docs/implementation/2026-10/2026-10-31-cola-0012-generate-schema.md`
- Design: `docs/design/2026-10-31-schema-dialect.md` ("Generated code")
- Spec: `docs/spec/schema.md` section 7
- Task: `docs/tasks/2026-10/2026-10-29-schema-validation.md` (done)
