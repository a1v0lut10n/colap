---
date: 2026-10-31
type: design
status: accepted
components: []
aspects: []
tags: [language-design, schema, validation, time]
---

# The schema dialect: declaring a document's shape in cola

## Context

A cola document is checked for syntax, duplicate keys and dangling
references, and nothing else. A field with a typo is accepted, a
value of the wrong type is accepted, a required field left out is
accepted, and the generated Rust type of a field is guessed from the
one literal the sample happens to hold. The field test's knowledge
repository wrote its own linter for the checks that belong in colap:
required fields, closed vocabularies, typed references, dates. The
schema task (`docs/tasks/2026-10/2026-10-29-schema-validation.md`)
asks for a schema dialect written in cola, a validator behind
`colap check --schema`, and a generator that types from declarations.

The value-language record deferred three things to schemas: typed
date and instant values, nulls, and decimals kept as text. The
exact-numbers record (`docs/design/2026-10-31-exact-numbers.md`)
answers the third in the model itself: a number is kept exactly as
written, and the schema only says which machine type a consumer gets. The
temporal-values incubation entry
(`docs/incubation/2026-10-30-temporal-values.md`) worked out what a
knowledge graph needs from time: transaction time as UTC instants,
valid time as civil dates that carry their precision, and the two
never sharing a type.

This record decides the dialect, the checks, and how the two are
bound. The validator and the generator follow in their own plans.

## Decisions

### A schema is a cola document

A schema is written in cola, so everything colap already does applies
to it: `check`, `fmt`, comments, the highlighter, and prose around the
declarations. A schema may be literate: the registry's documentation
and its schema can be one file. There is no second syntax to learn
and no second parser to maintain.

### The schema mirrors the document

The schema is a tree of entity declarations nested the way the
document's entities nest. Each declaration is keyed by the document
entity's singular name, under a collection `entity plural entities`;
a `plural` field marks a collection and the declaration then
describes each entry. The parser checks the structure, a renamed
entity is a visible change at one place, and reading the schema
top-down reads the document's shape.

```cola
schema:
    name: "llm-registry",
    version: 1,
    entity plural entities:
        llm:
            plural: llms,
            key_pattern: "[a-z][a-z0-9-]*",
            field plural fields:
                name: type: string, required: true;
                max_input_tokens: type: integer, min: 1;
                output_price: type: float, doc: "USD per million tokens";
                supports_vision: type: boolean;
                basis: type: symbol, one_of: [mirrored, parsed, derived];
                tags: type: symbol, list: true;
                vendor: type: reference, to: "vendor";
                recorded_at_utc: type: instant, required: true;
            ;
            entity plural entities:
                api:
                    required: true,
                    field plural fields:
                        type:
                            type: symbol,
                            one_of: [REST, gRPC],
                            required: true,
                        ;
                        key: type: string, pattern: "[A-Za-z0-9_-]+";
                    ;
                ;
            ;
        ;
        vendor:
            plural: vendors,
            unknown: allow,
            field plural fields:
                name: type: string, required: true;
                founded: type: date;
                active: type: edtf, doc: "valid time, from sources";
            ;
        ;
    ;
;
```

The alternative, a flat list of declarations keyed by path strings
(`"llm/api"`), is shorter for deep trees but the parser cannot check
the strings and a rename breaks them silently.

### Declarations

The root entity is `schema`, with optional fields `name` (string),
`version` (integer) and `utc_suffix` (boolean, default `true`, see
below).

An **entity declaration** has these fields, all optional:

| Field | Type | Meaning |
|---|---|---|
| `plural` | symbol | The collection's plural name. Present: the document entity is `name plural plural` and the declaration describes each entry. Absent: a singular slot. |
| `required` | boolean | The entity must be present in its parent. Default `false`. |
| `unknown` | symbol | What to do with a field or child entity the declaration does not name: `error` (default), `warn`, `allow`. |
| `key_pattern` | string | A regular expression every entry key must match in full. Collections only. |
| `like` | reference | Another declaration in this schema whose fields and children this one has; it then declares none of its own. For shared shapes, and for trees (a declaration that contains itself). |
| `doc` | string | Documentation, carried into generated code. |

and two collections: `field plural fields`, keyed by field name, and
`entity plural entities`, keyed by child entity name.

A **field declaration** has these fields, all optional:

| Field | Type | Meaning |
|---|---|---|
| `type` | symbol | One of the types below. Default `any`. |
| `list` | boolean | The field holds a list of `type`. Default `false`. |
| `required` | boolean | The field must be present. Default `false`. |
| `one_of` | list | The admissible values, as literals of the field's type. |
| `min`, `max` | number | Inclusive bounds, compared exactly, for the numeric types. |
| `pattern` | string | A regular expression the value must match in full, for `string` and `symbol`. |
| `to` | string | For `reference`: the path whose direct children are the admissible targets. |
| `doc` | string | Documentation, carried into generated code. |

The **types** and the document values they accept (section 5.5 of the
spec names the value kinds):

| Type | Accepts |
|---|---|
| `string` | a string |
| `integer` | an integer that fits 64 bits; generated as `i64` |
| `float` | a float, or an integer; generated as `f64` |
| `number` | a float or an integer; generated as `f64` |
| `decimal` | a float or an integer of any size, kept exact; generated as a `String` holding the canonical literal |
| `boolean` | `true` or `false` |
| `symbol` | a symbol |
| `reference` | a reference, which the parser has already resolved |
| `instant` | a string holding an RFC 3339 date-time with `Z` or an offset, a real calendar instant |
| `date` | a string holding an ISO 8601 date at year, month or day precision: `1914`, `1914-08`, `1914-08-04` |
| `datetime` | a string holding a civil date and time with no offset, `1919-06-28T15:12`, with seconds and fractions optional, and optionally an IANA zone in brackets as RFC 9557 spells it, `2026-10-30T09:00[Europe/Zurich]` |
| `edtf` | a string holding an EDTF level 0 or level 1 date, interval or season: `1914?`, `~1850`, `191X`, `1914/1918`, `1914/..` |
| `any` | anything |

A value is checked by its literal kind, the way the spec types values:
a `string` field does not accept a symbol and a `symbol` field does
not accept a string, because the two are different things in cola.
`float` accepts an integer literal, since `8` is a float the author
did not bother to write as `8.0`; `integer` is strict. The model holds
every number exactly, so the numeric types differ in what they admit
and in the generated type, never in what is read: `integer` rejects a
value past 64 bits because `i64` could not hold it, `decimal` admits
any and keeps it.

### Time

Two kinds of time, never one type. `instant` is transaction time: when
something was recorded, exact, with its offset. `date`, `datetime` and
`edtf` are valid time: when a fact held in the world, civil, at the
precision the source gives. A value with an offset is an instant; one
without is civil. The distinction is in the value and the type checks
it, so a `datetime` field holding `2026-10-30T09:00:00Z` is a finding.

The workspace's naming convention becomes checkable. When the schema
is loaded, a field whose name ends in `_utc` and whose type is not
`instant` is a schema error, and an `instant` field whose name does
not end in `_utc` is a schema warning. When a document is checked, an
instant in a `_utc` field written with an offset other than `Z` is a
warning: the value is correct, the convention is not. `utc_suffix:
false` on the schema switches all three off, for documents that
follow another convention.

Canonical form never changes a temporal value: an offset stays as
written, since it is information about the source. Durations are
numbers with the unit in the field's name and need no type.

### Checks

`colap check --schema S.md FILE...` validates each file against the
schema, on its own, after the ordinary checks. A file that fails to
parse or build is not validated. The findings, all reported, with the
location of the field, entity or key concerned:

| Finding | Severity |
|---|---|
| a required entity or field is missing | error |
| a field or entity the declaration does not name | per `unknown` |
| the document's declaration differs: singular where the schema says plural, a different plural name | error |
| a value of the wrong kind, or a scalar where a list is declared, or the reverse | error |
| a value outside `one_of`, `min`, `max` or `pattern`, or an `integer` past 64 bits | error |
| an entry key outside `key_pattern` | error |
| a reference whose target is not a child of `to` | error |
| a string that is not a valid `instant`, `date`, `datetime` or `edtf` | error |
| an instant in a `_utc` field not written with `Z` | warning |

Messages name the entity by its path and the value as it is written:
``field `max_input_tokens` in `llm/openai` must be an integer, got
"many"``, ``value derived2 of `basis` in `llm/openai` is not one of
[mirrored, parsed, derived]``, ``missing required field `name` in
`llm/openai```, ``unknown field `nmae` in `llm/openai```, ``reference
`@llm/openai` in `vendor` must point to an entry of `vendor```.

Errors and warnings go through the diagnostics channel of section 6
of the spec, in the same textual form, and `check` exits 1 on any
error, as it does today. Warnings never fail a check.

### The schema is checked by a schema

The dialect above is itself declared in a schema, the meta-schema,
shipped inside the crate. A declaration contains declarations, which
the mirrored tree can only say by naming one: `like` was added to the
dialect for this (planning note in the COLA-0011 plan), and serves
any document whose shape recurs or repeats. Loading a schema validates it against the
meta-schema with the same validator, so a misspelled `requried: true`
or a `type: strnig` is reported at its line before any document is
checked. The meta-schema validates itself; that is a test.

### Nulls

A null is an optional field left out: `required: false` is the whole
of it, and there is no null literal.

### Binding

A document is bound to its schema on the command line: `colap check
--schema S.md FILE...` and, later, `colap generate --schema S.md`.
Nothing in a document names its schema. An in-band declaration was
considered and deferred: a reserved root entity would reserve the
language's first name and sit in every model, and a path inside
content is fragile. When an editor needs to find the schema on its
own, the place for it is the literate document's frontmatter
(`cola-schema: path`), which is metadata and touches neither the
model nor the language; that is the path to take, then.

### Generated code

With `--schema`, the generator types from declarations instead of
sample literals: a required field is `T`, an optional one
`Option<T>`, a list `Vec<T>`, a `symbol` with `one_of` a Rust enum
named after the field, `reference` a `String` path, the temporal
types `String` (typed values behind a feature flag, with `jiff`, when
wanted), `decimal` the exact `Number` (decided `String` here, changed
in the COLA-0012 plan once the type existed), `any` a `ConfigValue`;
`doc` becomes a doc comment. A
collection's entry type comes from its declaration, so an entry that
happens to hold `1` no longer makes the field `i64` forever. The
generator ticket decides the rest.

## Consequences

- A new module, `schema`, with the meta-schema as a cola document in
  the crate; `check` gains `--schema`; the schema dialect gets its own
  spec document beside `docs/spec/cola.md`, literate and checked; the
  README documents schemas and the spec's section 9 loses its last
  open decision.
- `regex`, already in the dependency tree through rustemo, becomes a
  direct dependency for `pattern` and `key_pattern`; `chrono`, already
  a dependency, parses instants. EDTF and reduced-precision dates are
  a small parser of their own, since no crate in the tree reads them.
- The validator reports through `Diagnostic`, so editors and
  downstream linters see schema findings the way they see parse
  errors. aicogito and the knowledge repository's linter can drop the
  generic checks once their documents carry schemas.
- Three tickets after the exact-numbers record lands: the validator
  with `check --schema` and the spec (this ticket), the generator's
  `--schema`, and the temporal literal if colap itself must ever
  order dates, which nothing here requires.
- Deferred: in-band binding through frontmatter; merging several
  files into one model before validation; `min_entries` and string
  lengths; declarations that add to another (`extends`; `like` only
  reuses one); typed
  temporal values in generated code; exact rationals in generated
  code behind a feature flag.
