---
date: 2026-10-29
type: spec
status: accepted
components: []
aspects: []
---

# The cola language

This document is the normative description of cola as implemented by colap
0.4. Where the README and this document disagree, this document is right
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
- **Punctuation**: `:` after a name, `,` after a field, `;` ending an
  entity, `[` and `]` around a list.
- **String**: text between double quotes, `"…"`, or single quotes, `'…'`,
  which may span lines. A backslash starts an escape: `\"`, `\'`, `\\`,
  `\n`, `\r`, `\t`, and `\u{…}` with one to six hex digits. Any other
  escape is an error. The value is the decoded text.
- **Block string**: text between `"""` and `"""`, for prose. No escape is
  decoded and a `"` inside is literal. The value is the text with its
  first line dropped when blank, its last line dropped with the newline
  before it when blank, and the longest common leading whitespace of the
  non-blank lines removed from every line.
- **Number**: `[+-]?[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9]+)?`.
  Underscores separate digits and are dropped. The value is the exact
  decimal the literal writes, of any size: no digit is rounded or lost,
  and `0.1` is a tenth. A literal with a decimal point or an exponent is
  of float kind, any other of integer kind; `8` and `8.0` are the same
  value of different kinds. There is no leading `.`.
- **Boolean**: `true` or `false`.
- **Symbol**: an identifier in value position, `basis: parsed`. `plural`
  is a symbol there, since it is a keyword only where a keyword can stand.
- **Reference**: `@` followed by identifiers joined by `/`:
  `@claims/cl-1`. Section 5.5 says what it must name.
- **Comment**: `//` to the end of the line. A comment is layout, like
  whitespace: it may stand wherever whitespace may, is never a value,
  and `//` inside a string is text. Prose outside a block is not a
  comment; it is prose. Section 5.6 says where a comment belongs.

```cola
// A comment on its own line.
lexical: // and one after the header
    name: 'single-quoted',
    quoted: "say \"hi\"",
    gloss: """
        Two lines of prose,
        indented as the author likes.
        """,
    ratio: 0.75,
    count: -3,
    big: 1_000_000,
    small: 1.5e-3,
    on: true,
    basis: parsed,
    see: @lexical,
    url: "http://example.org", // the `//` in the string is text
    tags: [alpha, "two words", 3]
;
```

## 4. Grammar

In EBNF, with the terminals of section 3:

```text
Block        = { Entity } ;
Entity       = Plural | Singular ;
Plural       = Identifier "plural" Identifier ":" Definition ";" ;
Singular     = Identifier ":" Definition ";" ;
Definition   = { Field [ "," ] | Entity } ;
Field        = Identifier ":" Value ;
Value        = Scalar | List ;
Scalar       = String | BlockString | Number | Boolean | Symbol | Reference ;
List         = "[" [ Scalar { "," Scalar } [ "," ] ] "]" ;
```

A definition is any interleaving of fields and nested entities. A comma
may follow a field, and that is how fields are separated: one comma, and
a trailing one before `;` or before a nested entity is allowed. A comma
never follows an entity. The reader also accepts two fields with no
comma between them; the canonical layout (section 7) always writes the
comma. A list holds scalars, never another list, and may be empty or end
in a comma. An entity may be empty (`name:;`), which a reader reports as
a warning.

```cola
grammar plural grammars:
    nested:
        a: 1, b: 2,
        inner: c: 3;
        d: [4, 5,],
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

A field's value is a number, a boolean, a string, a symbol, a reference,
or a list of those, as section 3 lexes them. Type is decided by the
literal alone: `8` is a number of integer kind, `8.0` and `8e0` are
numbers of float kind, `"parsed"` is a string and `parsed` a symbol.
Numbers compare by value, so `1.10`, `1.1` and `11e-1` are equal, and
each keeps its spelling. A reader that wants a machine type takes it
from the number: a 64-bit integer when the value is integral and fits,
the nearest double, or the digits and their power of ten for exact
arithmetic (`docs/design/2026-10-31-exact-numbers.md`).

A reference's path names an entity from the root the way section 5.2
names it: each segment is the singular or plural name of a plural, the
key of an entry, or the slot of a singular entity. Every reference in a
document, in a list or not, must resolve in the finished model; one that
does not is an error at its field. A reference may point forward, and
into another block of the same document.

### 5.6 Comments

The model keeps every comment, attached to an item by position, so that
a formatter can write it back where it was. A comment on the line where
a field, an entity header or an entity's `;` ends is that item's
**trailing** comment; when several items end on that line, the last one
takes it. Any other comment is a **leading** comment of the next field
or entity among the direct items of the innermost entity around it;
when no item follows, it leads that entity's `;`; a comment after the
last entity of a block, or in a block with none, belongs to the end of
the block. A comment on a line inside a multi-line list, neither on the
list's last line nor before the field, is a leading comment of the
field. The text is kept as written after `//`, trailing whitespace
trimmed.

```cola
// leads `comment`
comment: // trails the header
    a: 1, // trails `a`
    // leads `b`
    b: 2,
    // leads the `;`
; // trails the `;`
// ends the block
```

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
`duplicate entry `cl-1` in `claim`; first at 5:5`. An unknown escape
reads `unknown escape `\q`` and a reference to nothing `unresolved
reference `@claims/cl-9``.

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
- an entity with no children whose one-line form fits in 80 columns and
  holds no multi-line string is written on one line:
  `name: a: 1, b: "x";`. Otherwise its header stands alone, each field on
  its own line ending in `,`, the last included, so that appending a field
  never touches the line before it; child entities follow, and `;` closes
  on its own line at the header's indentation;
- fields come before child entities, both in source order;
- a number is its literal with digit separators, a leading `+` and
  leading zeros of the integer part dropped and the exponent letter in
  lower case, nothing else changed: `007` is `7`, `1.10` stays `1.10`
  and `1e3` stays `1e3`; booleans are `true` and
  `false`; a symbol is bare and a reference is `@path`;
- a list is written `[a, b, c]` on its field's line when that line fits in
  80 columns, otherwise one element per line, each ending in `,`, one
  level deeper, with `]` at the field's indentation; an empty list is
  `[]`;
- a string containing a newline is written as a block string whose lines
  stand one level deeper than its field and whose closing `"""` stands
  at the field's indentation, when that form reads back as the same
  value; any other string is on one line, double-quoted with `\\`, `\"`,
  `\n`, `\r`, `\t` and control characters escaped, except that a string
  containing `"` and no `'` is single-quoted;
- a leading comment stands on its own line at its item's indentation, a
  trailing comment follows the item on its line, after the field's `,`,
  the header's `:` or the `;`, one space away, as `//` and the text as
  kept; an entity whose header or fields carry a comment, or that has
  a comment before its `;`, is never written on one line, while a
  comment after the `;` is written after the one-line form; the
  comments at the end of a block follow its last entity.

Formatting a literate document rewrites only the bodies of its cola
fences, each block on its own; prose and foreign fences are untouched, and
a collection split over several blocks stays split. A document with
errors is not formatted.

```cola
canonical:
    short: a: 1, b: "x";
    long:
        first: "a value that is long enough to push this entity past the width",
        second: 2,
        gloss: """
            Two lines
            of prose.
        """,
    ;
    // leads `noted`
    noted: a: 1; // trails the one-line form
;
```

## 8. Generated code

A code generator maps the model to types: a struct per singular entity, a
collection type per plural entity with lookup by key and iteration in
source order, and fields typed by their literal — `i64`, `f64`, `bool`,
`String`. Type inference from literals is the current behaviour; a
generator reading declarations from a schema is planned.

## 9. Schemas

A **schema** is a cola document declaring the shape other documents must
have: which entities and fields, of which types, required or not, closed
sets, bounds, patterns, typed references, instants and civil dates. A
reader validates a document against a schema on request (`colap check
--schema S.md FILE...`) and reports findings through the channel of
section 6. The dialect is specified in `docs/spec/schema.md`.

There is no null literal: an optional field left out is the null. Typed
temporal literals are not part of the language; a schema types a string
field as an instant or a date instead.

## 10. The conformance corpus

`tests/conformance/valid/` holds documents a reader must accept with no
error; `tests/conformance/invalid/` holds documents it must reject. Beside
any file, `NAME.expected` lists the diagnostics the reader must produce,
one per line in the textual form of section 6, with `<path>` standing for
the file's path; no sidecar means no diagnostics. The corpus is seeded with
every reproduction from the 2026-10-29 field test.

The test runner (`tests/conformance.rs`) checks every file against its
sidecar, checks that this document and the schema spec are clean, checks
the schema corpus (`docs/spec/schema.md`, section 7), and checks that the
editor highlighter's identifier, symbol, reference, number and string
tokens on every cola line are exactly the parser's located terminals.
