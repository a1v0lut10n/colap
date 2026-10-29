---
date: 2026-10-29
type: spec
status: accepted
components: []
aspects: []
---

# The cola language

This document is the normative description of cola as implemented by colap
0.3. Where the README and this document disagree, this document is right
and the README is a bug. It is itself a literate cola document: every
```` ```cola ```` block below is valid, and `colap check docs/spec/cola.md`
is clean. Examples that show what is *not* allowed sit in `text` fences.

The conformance corpus under `tests/conformance/` holds the parser, the
diagnostics and the editor highlighter to this document (section 10).

## 1. Scope and conformance

cola is a configuration language for literate documents: a markdown file
in which fenced `cola` blocks carry structured data and everything else is
prose. A *reader* is conforming when it accepts every file in the valid
corpus with exactly the stated warnings, rejects every file in the invalid
corpus with exactly the stated diagnostics, and builds the model section 5
describes. A *writer* is conforming when it produces the canonical layout
of section 7.

Terms used throughout: **document** (a whole file), **block** (the body of
one cola fence), **entity**, **field**, **key** and **slot** (section 5).

## 2. Documents

### 2.1 Literate documents

A literate document is markdown. Its cola is the bodies of its fenced code
blocks whose language is `cola`; the rest of the document is never read.
Prose lines are unconstrained: headings, tables, blockquotes, HTML, lines
starting with inline code or a bare `#`, and fenced blocks in any other
language are all fine.

Fences follow CommonMark (§4.5 of the spec):

- an **opening fence** is a run of three or more backticks or tildes,
  indented by at most three spaces, followed by an *info string*. The
  first word of the info string is the language; the rest is ignored. A
  backtick fence's info string may not contain a backtick (that line is an
  inline code span, not a fence);
- a **closing fence** uses the same character, is at least as long as the
  opener, is indented by at most three spaces, and carries nothing but
  blanks after it. A longer fence may therefore quote a shorter one;
- a fence that is never closed runs to the end of the document. A reader
  reports it as a warning naming the opening line and still reads the body;
- body lines are taken as written, newlines included. A reader does not
  strip the opener's indentation (cola ignores whitespace; keeping the
  bytes keeps columns exact).

```cola
service:
    name: "spec-example",
    port: 8080
;
```

Blocks are read in document order and contribute to one model (section
5.4). A document with no cola block at all yields an empty model and a
warning.

### 2.2 Bare documents

A bare document is cola from the first byte to the last, with no fences:

```text
service:
    name: "spec-example"
;
```

### 2.3 Which is which

A file named `.md` or `.markdown` is literate. For any other file, and for
text whose origin is unknown, a reader uses detection: the text is literate
if it contains at least one cola fence, bare otherwise. Markdown without a
cola fence read through detection is therefore a parse error, not a
silently empty model.

## 3. Lexical structure

Inside a block, whitespace — spaces, tabs, newlines, carriage returns — is
insignificant except as a separator. Indentation carries no meaning.

- **Identifier**: `[a-zA-Z_][a-zA-Z0-9_.-]*`. Dots and hyphens are allowed
  inside, so `gpt-4.1` is one identifier.
- **Keywords**: `plural`, `true`, `false`. The parser is state-driven, so a
  keyword is recognised only where a keyword can stand; elsewhere the same
  spelling is an identifier. Using keywords as names is accepted and not
  recommended: editor highlighting shows them as keywords.
- **Punctuation**: `:` after a name, `,` between fields, `;` ending an
  entity.
- **String**: text between double quotes, `"…"`, or single quotes, `'…'`.
  A backslash escapes the next character for the purpose of finding the
  closing quote; the text between the quotes is kept **as written**,
  escapes included, and may span lines. Decoding escapes is an open
  decision (section 9).
- **Number**: `[+-]?[0-9]+(\.[0-9]+)?`. With a decimal point the value is
  a 64-bit float; without, a 64-bit signed integer, and a digit string
  outside that range is an error. There is no exponent form.
- **Boolean**: `true` or `false`.
- There are no comments.

```cola
lexical:
    name: 'single-quoted',
    quoted: "say \"hi\"",
    ratio: 0.75,
    count: -3,
    on: true
;
```

## 4. Grammar

In EBNF, with the terminals of section 3:

```text
Block        = { Entity } ;
Entity       = Plural | Singular ;
Plural       = Identifier "plural" Identifier ":" Definition ";" ;
Singular     = Identifier ":" Definition ";" ;
Definition   = { FieldList | Entity } ;
FieldList    = Field { "," Field } ;
Field        = Identifier ":" Value ;
Value        = String | Number | Boolean ;
```

A definition is any interleaving of field lists and nested entities. A
comma separates fields within one list; there is no comma after the last
field of a list, and none between a field and an entity. An entity may be
empty (`name:;`), which a reader reports as a warning.

```cola
grammar plural grammars:
    nested:
        a: 1, b: 2
        inner: c: 3;
        d: 4
    ;
;
```

## 5. The model

### 5.1 Entities and fields

A document denotes a tree of **entities** under an unnamed root. An entity
has a name, an ordered list of **fields** (name and value) and an ordered
list of child entities. A **plural** entity, declared `name plural names:`,
is a collection: its singular name is used in paths and generated types,
its plural name for the collection. Every other entity is **singular**.

Fields and children keep their source order, across blocks (section 5.4).

### 5.2 Keys and slots

A name directly under a plural is a **key**: it identifies an entry of the
collection. Every other entity name is a **slot**: it names a place in the
structure. This is the distinction the `plural` keyword draws, and the
rules below follow it. (Design record: `docs/design/2026-10-29-duplicate-key-semantics.md`.)

### 5.3 Rules

1. **Keys are defined once.** Two entries with the same key under the same
   plural are an error, in one block or across blocks. The second
   definition is reported with both locations and its subtree is dropped.
2. **Slots may be reopened.** The same declaration at the same path
   continues the same entity: a reopened plural appends entries, a
   reopened singular adds fields and nested slots. The entity keeps the
   location of its first opening. Reopening with a different declaration —
   another plural name, or singular where it was plural and vice versa — is
   an error with both locations.
3. **Fields are set once.** A field name repeated in one entity, across
   reopenings or not, is an error with both locations. There is no
   override.
4. **One namespace per entity.** Within an entity a name is a field or an
   entity, never both, and a plural declaration reserves both its singular
   and its plural name. Clashes are errors with both locations.

Names are compared exactly and case-sensitively. A **path** is the chain
of singular names from the root, joined by `/`; a collection may also be
addressed by its plural name.

A consequence of rule 1: a key cannot be reopened, so entries of a
collection nested under a key are written in one block.

### 5.4 Several blocks, several files

The blocks of a document are read in order into one model, and the rules
above apply across them. A reader that accepts several documents at once
reads them in the order given, the same way.

```cola
claim plural claims:
    cl-1: subject: "first";
;
```

Prose may separate the batches of one collection:

```cola
claim plural claims:
    cl-2: subject: "second";
;
```

That document denotes one collection `claim` with entries `cl-1` and
`cl-2`, in that order. A third block defining `cl-1` again would be an
error.

### 5.5 Values

A field's value is an integer, a float, a boolean or a string, as section
3 lexes it. A string's value is the raw text between its quotes. Type is
decided by the literal alone: `8` is an integer and `8.0` a float.

## 6. Diagnostics

A reader reports **every** finding in a document, not only the first: a
block that fails to parse does not hide the blocks after it, and every
duplicate is reported. Findings are ordered by line.

Each finding has a severity (`error` or `warning`), a path, a 1-based line
and column in the original document where it has one, and a message.
The textual form is:

```text
path:line:col: severity: message
path: severity: message            (no location)
```

A duplicate's message names the entity and ends with the first location:
`duplicate entry `cl-1` in `claim`; first at 5:5`.

Warnings a reader must give:

- a literate document with no cola block;
- an unclosed fence (naming its opening line);
- an empty entity.

A command-line reader exits 0 when no error was found (warnings allowed),
1 when any error was found, and 2 for a usage error.

## 7. Canonical layout

There is one canonical way to write a model, so that files regenerated by
tools are byte-deterministic and formatting is idempotent. The layout is a
pure function of the model:

- four spaces of indentation per nesting level;
- an entity with no children whose one-line form fits in 80 columns is
  written on one line: `name: a: 1, b: "x";`. Otherwise its header stands
  alone, each field on its own line with `,` after all but the last, child
  entities follow, and `;` closes on its own line at the header's
  indentation;
- fields come before child entities, both in source order;
- an integer is written in decimal without a sign for non-negative values;
  a float always carries a decimal point; booleans are `true` and `false`;
- a string is double-quoted unless its raw text contains an unescaped
  double quote, in which case it is single-quoted. The raw text is never
  changed.

Formatting a literate document rewrites only the bodies of its cola
fences, each block on its own; prose and foreign fences are untouched, and
a collection split over several blocks stays split. A document with
errors is not formatted.

```cola
canonical:
    short: a: 1, b: "x";
    long:
        first: "a value that is long enough to push this entity past the width",
        second: 2
    ;
;
```

## 8. Generated code

A code generator maps the model to types: a struct per singular entity, a
collection type per plural entity with lookup by key and iteration in
source order, and fields typed by their literal — `i64`, `f64`, `bool`,
`String`. Type inference from literals is the current behaviour, not a
commitment; schemas are an open decision (section 9).

## 9. Open decisions

The following are not part of the language as specified. Each has a task
and will be decided in a design record; this document is updated when a
decision is accepted.

- **Value language**: bare symbols, lists of scalars, references,
  comments, decoded string escapes and raw or indented multi-line strings,
  exponents and digit separators in numbers, typed date and instant
  literals, trailing commas
  (`docs/tasks/2026-10/2026-10-29-value-language-extensions.md`).
- **Schemas**: declaring the shape a document must have, and checking
  against it (`docs/tasks/2026-10/2026-10-29-schema-validation.md`).

Until then: there are no trailing commas, no comments, and escapes are
kept raw.

## 10. The conformance corpus

`tests/conformance/valid/` holds documents a reader must accept with no
error; `tests/conformance/invalid/` holds documents it must reject. Beside
any file, `NAME.expected` lists the diagnostics the reader must produce,
one per line in the textual form of section 6, with `<path>` standing for
the file's path; no sidecar means no diagnostics. The corpus is seeded with
every reproduction from the 2026-10-29 field test.

The test runner (`tests/conformance.rs`) checks every file against its
sidecar, checks that this document is clean, and checks that the editor
highlighter's identifier, number and string tokens on every cola line are
exactly the parser's located terminals.
