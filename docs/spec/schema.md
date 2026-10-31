---
date: 2026-10-31
type: spec
status: accepted
components: []
aspects: []
---

# The cola schema dialect

This document is the normative description of cola schemas as
implemented by colap 0.4. A schema is a cola document that declares
the shape other cola documents must have; `colap check --schema S.md
FILE...` validates each file against it. Where the README and this
document disagree, this document is right. It is itself a literate
cola document: every ```` ```cola ```` block below is valid cola and
`colap check docs/spec/schema.md` is clean. The language itself is
specified in `docs/spec/cola.md`, whose terms this document uses.

The design record is `docs/design/2026-10-31-schema-dialect.md`; the
corpus under `tests/conformance/schema/` holds the validator to this
document (section 8).

## 1. A schema is a cola document

Everything colap does for a document applies to a schema: `check`,
`fmt`, comments, the highlighter, prose around the declarations. A
schema may be literate; this document is one.

A schema document has one root entity, `schema`, with optional fields
`name` (a string), `version` (an integer, at least 1), `utc_suffix` (a
boolean, default `true`, section 4) and `unknown` (section 3, for the
root), and a collection `entity plural entities` of entity
declarations, one per root entity of the documents it describes.

```cola
schema:
    name: "example",
    version: 1,
    entity plural entities:
        database:
            field plural fields:
                url: type: string, required: true;
            ;
        ;
    ;
;
```

That schema accepts a document holding `database: url: "x";` and
nothing else.

## 2. Declarations

The schema mirrors the document: entity declarations nest the way the
document's entities nest, each keyed by the document entity's
singular name.

### 2.1 Entity declarations

An entity declaration is an entry of an `entity plural entities`
collection. Its fields, all optional:

| Field | Type | Meaning |
|---|---|---|
| `plural` | symbol | The document entity is `name plural plural`, a collection, and the declaration describes each entry. Absent: a singular entity. |
| `required` | boolean | The entity must be present in its parent. Default `false`. |
| `unknown` | symbol | What to do with a field or child entity the declaration does not name: `error` (default), `warn`, `allow`. |
| `key_pattern` | string | A regular expression every entry key must match in full. Collections only. |
| `like` | reference | Another entity declaration in this schema; this declaration has its fields and child entities and declares none of its own. |
| `doc` | string | Documentation, carried into generated code. |

and two collections: `field plural fields` of field declarations
(2.2), and `entity plural entities` of child entity declarations.

`like` serves shapes that repeat, and trees: a declaration may name
the declaration it sits in. The reference is a cola reference,
resolved by the parser; its path names the declaration the way
section 5.5 of the language spec names entities (`@schema/entity/
folder` is the declaration `folder` under `schema`).

```cola
schema:
    entity plural entities:
        folder:
            plural: folders,
            field plural fields:
                name: type: string, required: true;
            ;
            entity plural entities:
                folder: plural: folders, like: @schema/entity/folder;
            ;
        ;
    ;
;
```

### 2.2 Field declarations

A field declaration is an entry of a `field plural fields`
collection, keyed by the field's name. Its fields, all optional:

| Field | Type | Meaning |
|---|---|---|
| `type` | symbol | One of the types of section 3. Default `any`. |
| `list` | boolean | The field holds a list of `type`. Default `false`. |
| `required` | boolean | The field must be present. Default `false`. |
| `one_of` | list | The admissible values, literals of the field's type. |
| `min`, `max` | number | Inclusive bounds, compared exactly. Numeric types only. |
| `pattern` | string | A regular expression the value must match in full. `string` and `symbol` only. |
| `to` | string | For `reference`: the path whose direct children are the admissible targets. |
| `doc` | string | Documentation, carried into generated code. |

A constraint that does not fit the type (`pattern` on an integer,
`min` on a string, `to` on anything but a reference, a `one_of` entry
of another kind, `min` above `max`, a pattern that does not compile,
`key_pattern` on a singular declaration, `like` beside fields or
entities of its own, `like` to anything but a declaration) is an
error when the schema is loaded.

## 3. Types

A value is checked by its literal kind, as the language spec types
values: a `string` field does not accept a symbol, nor a `symbol`
field a string.

| Type | Accepts |
|---|---|
| `string` | a string |
| `integer` | a number of integer kind that fits 64 bits |
| `float` | a number of float kind, or of integer kind |
| `number` | a number of either kind |
| `decimal` | a number of either kind and any size, kept exact |
| `boolean` | `true` or `false` |
| `symbol` | a symbol |
| `reference` | a reference, which the parser has resolved; with `to`, one whose target is a direct child of the entity at that path |
| `instant` | a string holding an RFC 3339 date-time with `Z` or an offset, naming a real moment |
| `date` | a string holding `YYYY`, `YYYY-MM` or `YYYY-MM-DD`, a real date at its precision |
| `datetime` | a string holding `YYYY-MM-DDThh:mm`, seconds and a fraction optional, no offset, optionally followed by an IANA zone in brackets, `[Europe/Zurich]`, checked for shape |
| `edtf` | a string holding an EDTF level 0 or level 1 date, date-time or interval (section 4) |
| `any` | anything, a list included |

With `list: true` the field must hold a list, possibly empty, and
every element is checked as above; without it a list is a finding,
except for `any`.

`unknown` on the `schema` entity says what to do with a root entity
the schema does not name; on an entity declaration, with the
entity's own fields and children. Fields and child entities share
one namespace in cola, so one setting covers both.

## 4. Time

Two kinds of time, never one type. `instant` is transaction time:
when something was recorded, exact, with its offset. `date`,
`datetime` and `edtf` are valid time: when a fact held in the world,
civil, at the precision the source gives. A value with an offset is
an instant; one without is civil. A `datetime` holding
`2026-10-30T09:00:00Z` is therefore a finding, and so is an `instant`
holding `2026-10-30T09:00`.

The `_utc` naming convention is checked unless the schema says
`utc_suffix: false`:

- when the schema is loaded, a field whose name ends in `_utc` and
  whose type is not `instant` is an error, and an `instant` field
  whose name does not end in `_utc` is a warning;
- when a document is checked, an instant in a `_utc` field written
  with an offset other than `Z` is a warning: the value is right, the
  convention is not.

The EDTF forms accepted, levels 0 and 1 of the Library of Congress
specification:

| Form | Examples |
|---|---|
| date at year, month or day precision | `1914`, `1914-08`, `1914-08-04` |
| date-time with optional `Z` or offset | `1985-04-12T23:20:30`, `1985-04-12T23:20:30+04:00` |
| qualifier after the date: `?` uncertain, `~` approximate, `%` both | `1914?`, `1850~`, `1850%` |
| unspecified trailing digits | `191X`, `19XX`, `1914-XX`, `1914-08-XX` |
| season in place of a month | `2001-21` (spring) to `2001-24` (winter) |
| long or negative year | `Y170000002`, `Y-17000`, `-0999` |
| interval, with `..` for an open end and nothing for an unknown one | `1914/1918`, `1914/..`, `../1918`, `1914/`, `/1918` |

A date with every digit given must exist in the calendar
(`2023-02-29` is a finding); the qualifier stands after the date, not
before it.

## 5. Findings

`colap check --schema S.md FILE...` loads the schema, then validates
each file on its own, after the file's ordinary checks; a file that
does not parse or build is not validated. Findings go through the
diagnostics channel of the language spec's section 6, in the same
textual form, in line order among the file's other diagnostics;
`check` exits 1 on any error. A schema that fails to load is reported
alone and nothing is validated.

| Finding | Severity |
|---|---|
| a required entity or field is missing | error |
| a field or entity the declaration does not name | per `unknown` |
| the document's declaration differs: singular where the schema says plural, a different plural name | error |
| a value of the wrong kind, a scalar where a list is declared, or a list where a scalar is | error |
| an `integer` past 64 bits | error |
| a value outside `one_of`, `min`, `max` or `pattern` | error |
| an entry key outside `key_pattern` | error |
| a reference whose target is not a direct child of `to` | error |
| a string that is not a valid `instant`, `date`, `datetime` or `edtf` value | error |
| an instant in a `_utc` field not written with `Z` | warning |

Messages name the entity by its path and the value as it is written:
``missing required field `name` in `llm/openai```, ``unknown field
`nmae` in `llm/openai```, ``field `max_input_tokens` in `llm/openai`
must be an integer, got "many"``, ``value guessed of `basis` in
`llm/openai` is not one of [mirrored, parsed, derived]``, ``value
@llms/other of `vendor` in `llm/openai` must point to an entry of
`vendor```, ``key `Bad` in `llm` does not match pattern
`[a-z][a-z0-9.-]*```. A missing member is reported at the entity that
lacks it, every other finding at the field, entity or key concerned.

## 6. The meta-schema

The dialect is declared in a schema, the meta-schema, shipped inside
colap (`src/schema/meta.md`, `colap::schema::META`). Loading a schema
validates it against the meta-schema with the same validator, so a
misspelled `requried` or a `type: strnig` is reported at its line as
an ordinary finding, before any document is checked. The meta-schema
validates itself; it is the one place `like` is needed by the
dialect, since a declaration contains declarations.

## 7. Generated code

`colap generate --schema S.md [FILE]` emits Rust types from the
declarations rather than from a sample's literals. One struct per
shape, named after its declaration in PascalCase (prefixed with its
parent's name when two would collide); a collection wrapper per plural
declaration, named after the plural name, with lookup by key and
iteration in source order; a `Root` for the root shape; a `like`
declaration has its target's type, so a tree is `Folder` holding
`Folders` of `Folder`.

| Declaration | Rust |
|---|---|
| `string`, `symbol`, `reference`, `instant`, `date`, `datetime`, `edtf` | `String` |
| `integer` | `i64` |
| `float`, `number` | `f64` |
| `decimal` | `colap::config_model::Number`, the exact value |
| `boolean` | `bool` |
| `any` | `Option<colap::config_model::ConfigValue>`, always optional |
| `symbol` with `one_of` | an enum named after the struct and the field, `LlmBasis { Mirrored, Parsed, Derived }`, with `as_str`, `FromStr` and the first symbol as default |
| `list: true` | `Vec<T>` of the above |
| a field not `required` | `Option<T>` |
| a child entity not `required` | `Option<Struct>`; a collection is always its wrapper, possibly empty |
| `doc` | a doc comment |

The generated code embeds the schema's text as `SCHEMA` and `Root`
offers `load(path)`, which reads the file, validates it against the
embedded schema and builds the root, or returns every error; and
`from_model(&model)` for a model the caller has already checked, where
a value the schema would reject is left at its default. The generated
crate depends on the published colap, or on a checkout given with
`--colap-path`; a document given with the schema is validated first,
copied into the crate's tests and loaded by its integration test.

## 8. The conformance corpus

`tests/conformance/schema/` holds schemas and documents in pairs:
`NAME.schema.md` is a schema, every `NAME-*.md` beside it a document
validated against it. `invalid-*.schema.md` are schemas that must fail
to load. Beside any file, `NAME.expected` lists the findings colap
must produce, one per line in the textual form of section 5, with
`<path>` standing for the file's path; no sidecar means no findings.
The test runner (`tests/conformance.rs`) checks every pair, checks
that the meta-schema validates itself, and checks that this document
is clean. `tests/generate_schema.rs` builds a crate generated from the
registry schema against the checkout and runs its tests.
