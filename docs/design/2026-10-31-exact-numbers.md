---
date: 2026-10-31
type: design
status: accepted
components: []
aspects: []
tags: [language-design, values, numbers, claimr]
---

# Numbers are exact decimals in the model; machine types are the consumer's choice

## Context

A number literal is converted when a block is built: with a decimal
point or an exponent it becomes an `f64`, otherwise an `i64`, and an
integer past 64 bits is an error. The canonical layout then writes
the number back from the machine value. For everyday literals the
round trip is invisible, because Rust prints the shortest text that
reads back to the same double, but it is a conversion all the same:
`1.10` comes back as `1.1`, a twentieth significant digit is gone,
`0.1` is the nearest double and not a tenth, and
`123456789012345678901` is rejected.

claimr, the workspace's constraint logic language, is the consumer
that makes this visible. Its design record
(`docs/design/2026-08-17-exact-rational-arithmetic.md` there) makes
every number an exact arbitrary-precision rational, with no float
type anywhere: `18.5` denotes 37/2, `0.1` denotes 1/10, and
constraint solving is sound because equality is exact. The knowledge
repository's cola records are meant to become claimr facts. A number
that has been through an `f64` cannot be handed over exactly, and
re-lexing the source text in the bridge would put the fix in the
wrong layer.

The schema dialect (`docs/design/2026-10-31-schema-dialect.md`) adds
`min` and `max` bounds and a `decimal` type. Both want an exact value
to compare and to keep. This record settles the model's number before
the validator is built on it.

## Decision

### One number, kept as written

The model has one numeric value, `ConfigValue::Number`, holding an
**exact decimal**: the sign, the significand's digits as written, the
length of the fraction and the exponent. No digit is dropped, rounded
or reinterpreted, and there is no size limit; `integer out of range`
stops being an error of the language. `Integer(i64)` and `Float(f64)`
go.

The number keeps its **kind** from the literal, as the spec already
rules: a literal with a decimal point or an exponent is a float, any
other an integer. `8` and `8.0` are the same value of different kinds,
and the kind is what the schema's `integer` and `float` types and the
generator's inference look at.

**Equality and order are numeric.** `1.10`, `1.1` and `11e-1` are
equal and sort together; the spelling is kept beside the value and
compared by nothing. The schema's `one_of`, `min` and `max` compare
exactly.

### Views for consumers

The value offers `is_integer()` (the kind), `as_i64()` and `as_f64()`
(`None` when the value does not fit, or the nearest double, each
documented), and the pair a bignum consumer needs: the significand as
a decimal digit string and the power of ten it is scaled by. claimr's
`Number::from_literal` reads that pair, or the canonical text, without
loss. colap takes no bignum dependency: a digit string and an exponent
are the whole representation.

### Canonical form

The canonical layout writes a number as its literal with digit
separators dropped, a leading `+` dropped, leading zeros of the
integer part dropped (`007` is `7`, `0.5` keeps its zero) and the
exponent letter in lower case, and nothing else changed: `1.10` stays
`1.10`, since a trailing zero in the fraction may say how precise the
value is, `1e3` stays `1e3`, `1_000` becomes `1000`. The former rule that a float past twenty
characters is rewritten in exponent form goes with the `f64` it
served; there is no longer a machine value to render from.

### Generated code

Inference is unchanged: an integer literal makes the field `i64`, a
float `f64`, read through `as_i64` and `as_f64`. A value that does not
fit is an error of the reader that builds the typed struct, not of
the language. With a schema, `decimal` fields are generated as a
`String` holding the canonical text, or as an exact rational behind a
feature flag when a consumer wants arithmetic.

### Lexical form

Unchanged: `[+-]?[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9]+)?`. No
rational literal: `1/3` is a division claimr performs, not a number
cola writes; a document that needs a third writes `0.333...` at the
precision it means, or a string.

## Consequences

- `ConfigValue` loses two variants and gains one, a breaking change
  for every match on it: the generator templates, the canonical
  layout, the highlighter's parity test, and chaiss's `value_text`,
  whose migration plan (CHAI-0044) gains the arm.
- `tests/conformance/invalid/integer-overflow.md` becomes a valid
  document; the schema's `integer` type takes over the 64-bit check
  for typed consumers.
- Spec section 3 (the number's value), section 6 (the overflow
  message) and section 7 (the canonical number) change; the
  value-language record's number paragraph is superseded by this one.
- The scale benchmark should not move: a digit string costs what the
  literal's text cost.
- Its own ticket, landed before the schema validator so that bounds
  compare exactly from the start.
