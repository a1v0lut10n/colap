---
date: 2026-10-31
type: task
ticket: COLA-0010
status: in-progress
affects:
  - src/model/config_model.rs
  - src/model/literal.rs
  - src/model/model_builder.rs
  - src/model/canonical.rs
  - src/generator/generator_impl.rs
  - docs/spec/cola.md
  - README.md
  - CHANGELOG.md
tasks:
  - docs/tasks/2026-10/2026-10-29-schema-validation.md
---

# COLA-0010: numbers kept exactly as written

## Premise

`docs/design/2026-10-31-exact-numbers.md` replaces `Integer(i64)` and
`Float(f64)` with one `ConfigValue::Number` holding the literal
exactly, so that no digit is lost between a document and a consumer
such as claimr, and so that the schema validator
(`docs/design/2026-10-31-schema-dialect.md`) compares bounds exactly.
This ticket lands before the validator.

## Action items

1. **`Number`** in `src/model/literal.rs`: built from the grammar's
   literal with separators and a leading `+` dropped and `e` in lower
   case; `text()`, `is_integer()` (the literal's kind), `as_i64()`
   (`None` unless the value is integral and fits), `as_f64()` (the
   nearest double), `decimal()` (sign, significand digits, power of
   ten) for bignum consumers; `From<i64>` and `From<f64>`; `Display`
   is the text; `Eq`, `Ord` and `Hash` are numeric.
2. **Model**: `ConfigValue::Number(Number)`; the builder parses into
   it; `integer out of range` disappears.
3. **Canonical layout**: a number is its text; `float_text` and the
   twenty-character rule go.
4. **Generator**: `i64` for an integer kind, `f64` otherwise, read
   through `as_i64` and `as_f64` in the three templates.
5. **Spec, README, changelog**: sections 3, 5.5, 6 and 7; the README's
   type table; `Unreleased` gains the breaking change; the
   value-language record notes the supersession.
6. **Tests**: `integer-overflow.md` moves from the invalid corpus to
   the valid one as `big-numbers.md` with a value past 64 bits and a
   spelling the old `f64` round trip changed; `tests/values.rs` pins
   exactness, numeric equality and the views; the generated crate
   check in the generator tests covers the new template arms.

## Verification

- [ ] The three Rust gates are clean.
- [ ] Corpus, spec and token parity hold; every clean fixture still
      round-trips and `fmt` is idempotent.
- [ ] `1.10`, `1_000`, `1e3` and a 30-digit integer survive
      `check` and `fmt` unchanged in value and, bar separators, in
      spelling.
- [ ] `cargo bench --bench scale` does not move.
