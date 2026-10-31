---
date: 2026-10-31
type: journal
components: []
aspects: []
tags: [numbers, schema, claimr]
---

# COLA-0010: the schema dialect decided, numbers kept exactly as written

## Context

The schema task asked for a design record first. Drafting it raised
numbers: claimr, the workspace's constraint logic language, keeps
every number as an exact rational and the knowledge repository's cola
records are meant to become claimr facts, which an `f64` in between
cannot serve. Two records resulted, both accepted on 2026-10-31: the
schema dialect, and exact numbers, which lands first so that the
validator compares bounds exactly.

## Details

The schema dialect is cola: a `schema` entity whose nested
`entity plural entities` mirror the document's tree, a `plural` field
marking a collection and its entry shape, `field plural fields` with
`type`, `list`, `required`, `one_of`, `min`, `max`, `pattern`, `to`
and `doc`. Types cover the scalars, `decimal` for numbers of any size,
and four temporal types that keep transaction time (`instant`) and
valid time (`date`, `datetime`, `edtf`) apart, with the `_utc` naming
convention checked and switchable. Unknown members are errors unless
an entity says `unknown: allow`. A document is bound to its schema on
the command line; frontmatter binding is the recorded path if an
editor ever needs more. A meta-schema checks schemas with the same
validator.

Exact numbers: `ConfigValue::Number` replaces `Integer(i64)` and
`Float(f64)`. The value is the literal's exact decimal, of any size,
with its kind (integer or float) kept from the spelling; equality and
order are numeric, so `1.10` and `1.1` are one value spelled two ways,
and canonical form drops only separators, a leading `+` and leading
integer zeros. Readers take `as_i64`, `as_f64` or the digit string
and power of ten. `integer out of range` has left the language, and
the former overflow fixture is now a valid one. The generator still
infers `i64` and `f64`, reading through the views; its crate is
edition 2021, so the `i64` reader nests two `if let`s.

Breaking for every match on `ConfigValue`: chaiss's migration plan
gained the arm.

## Links

- Designs: `docs/design/2026-10-31-schema-dialect.md`,
  `docs/design/2026-10-31-exact-numbers.md`
- Plan: `docs/implementation/2026-10/2026-10-31-cola-0010-exact-numbers.md`
- Task: `docs/tasks/2026-10/2026-10-29-schema-validation.md`
- claimr: `docs/design/2026-08-17-exact-rational-arithmetic.md`
- Spec sections 3, 5.5, 6 and 7 in `docs/spec/cola.md`
