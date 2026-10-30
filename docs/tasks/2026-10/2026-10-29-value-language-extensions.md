---
date: 2026-10-29
type: task
status: done
affects:
  - README.md
tags: [language-design, grammar, values]
---

# Extend the value language: symbols, lists, references, comments, proper strings and numbers

## Objective

Decide in one design record, then implement, the value forms that using cola for a
literate knowledge repository showed it lacking. The language stays punctuation-oriented and whitespace-insensitive.

## Context

The registry, ledger and crosswalk files of a downstream knowledge repository had to
work around each of these gaps.

| Gap | Today | Workaround the downstream project adopted |
|---|---|---|
| Bare symbols | `UnquotedString` is defined in the grammar but never used, so every enum value is a quoted string | Quote everything; the validator cannot tell an enum token from free text |
| Lists of scalars | Only `plural` keyed maps exist | Space-separated token strings ("alpha beta gamma"), split by the consumer, or plural blocks with synthetic keys (l1, l2, ...) |
| References | No reference type; ids are strings | Strings like "entity:example-1", resolved and checked by a separate tool |
| Comments inside blocks | None; Layout is whitespace only | Move every explanation into surrounding prose, even per-entry notes |
| String escapes | Matched by the lexer, but `ModelBuilder` keeps the raw text between quotes, so a backslash-quote stays two characters | The downstream linter forbids backslashes and embedded double quotes |
| Multi-line text | Newlines inside quotes are kept, including the indentation | Accept the indentation noise in long glosses |
| Numbers | No exponent, no leading `.`; overflow of i64 is a parse error; no way to keep a decimal as text | Write 0.000012, never 1.2e-5 |
| Dates and instants | Strings only | RFC 3339 strings with naming conventions (`_utc`) |
| Empty values | No null or absent marker | Omit the field |

## Deliverables

1. **A design record** (`docs/decisions/ADR-NNN-value-language-extensions.md`) weighing
   each proposal for readability, ambiguity and backwards compatibility:
   - **Bare symbols** (`basis: parsed`), matching the identifier pattern, as a distinct
     `ConfigValue::Symbol`.
   - **Scalar lists** (`tags: [alpha, "two words", 3]`), or the infix style
     `tags: alpha | beta | gamma`, to match the `plural` idiom.
   - **References** (`subject: @entity:example-1`) as `ConfigValue::Reference`,
     resolvable within a model and by consumers.
   - **Comments.** A line comment form that cannot collide with markdown or values:
     `--` or `//` to end of line, inside cola fences only.
   - **Strings.** Escapes are unescaped (`\"`, `\\`, `\n`, `\t`, `\u{...}`); a raw or
     triple-quoted multi-line string strips common indentation.
   - **Numbers.** Exponents and digit separators are accepted; out-of-range values are an
     error rather than a panic.
   - **Typed literals,** optionally, for dates and instants (`2026-10-29T10:00:00Z`
     unquoted), or explicitly deferred to schemas.
   - **Optional trailing commas.** The README already claims "Trailing commas allowed:
     Yes", but the grammar has no such rule. Either implement them or correct the claim.
2. **Grammar, model, generator and highlighter changes** for the accepted subset, each
   with tests.
3. **Migration note.** Existing files stay valid, since every proposal above is additive.

## Explicitly out of scope

- Schema-level typing (`2026-10-29-schema-validation.md`). This task defines values, not
  constraints on them.

## Completion

Before setting `status: done`:
- the ADR is accepted;
- README.md's syntax section and comparison table match the grammar;
- a journal entry links back to this task.

## Progress note

Decided in `docs/design/2026-10-30-value-language.md` (accepted
2026-10-30). COLA-0007 implemented every value form: symbols, lists,
references, decoded escapes and block strings, number separators and
exponents, trailing commas. Comments followed in COLA-0009
(`docs/implementation/2026-10/2026-10-30-cola-0009-comments.md`):
found with the highlighter's lexer, attached to the model by position,
written back by `fmt`. Spec and README match the grammar; the journal
entries of 2026-10-30 link back here.
