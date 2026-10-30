---
date: 2026-10-30
type: design
status: accepted
components: []
aspects: []
tags: [language-design, grammar, values]
---

# The value language: symbols, lists, references, comments, decoded strings

## Context

cola 0.2 has four value forms: integer, float, boolean, and a string
whose text is kept raw between its quotes. A downstream knowledge
repository using the format for registries, ledgers and crosswalks had
to work around everything else: enum tokens were quoted strings the
validator could not tell from free text, lists were space-separated
tokens or plural blocks with synthetic keys, references were strings
resolved by a separate tool, per-entry notes had to move into the
surrounding prose, backslashes and embedded quotes were forbidden by
the downstream linter, and long glosses carried their indentation.

The task (`docs/tasks/2026-10/2026-10-29-value-language-extensions.md`)
asks for one record deciding each proposal. The constraints: the
language stays punctuation-oriented and whitespace-insensitive, every
addition is additive so existing files stay valid, the parser stays
LR(1), the canonical layout stays a function of the model, and the
highlighter can lex every form line by line.

## Decision

### Symbols

An identifier in value position is a **symbol**: `basis: parsed`,
`model: gpt-4.1`. `true` and `false` stay booleans; `plural` is a
symbol in value position, as the lexer is state-driven. The model gets
`ConfigValue::Symbol(String)`; generated code types it `String`; the
canonical form is bare; the highlighter shows it as a symbol, distinct
from a name.

### Lists

A list is `[` scalars separated by `,` `]`: `tags: [alpha, "two
words", 3]`, `none: []`. A trailing comma inside the brackets is
allowed. Elements are scalars (string, number, boolean, symbol,
reference); a list inside a list is a parse error. The model gets
`ConfigValue::List(Vec<ConfigValue>)`. Generated code types a list
`Vec<T>` from its first element, and `Vec<colap::ConfigValue>` when it
is empty or mixed. The canonical form is `[a, b, c]` on the field's
line when the entity's or field's line fits in 80 columns, otherwise
one element per line, one level deeper, each line ending in `,`, `]`
at the field's indentation.

Brackets over the infix `a | b | c` because a one-element list and a
symbol would otherwise read the same, and because `[]` names the empty
list.

### References

A reference is `@` followed by a path: `subject: @claims/cl-1`. The
path's segments are entity names as section 5 of the spec resolves
them from the root (a plural is reachable by its singular or plural
name; a key by its name); the segments are identifiers and the
separator is `/`, which no identifier contains. The model gets
`ConfigValue::Reference(String)` holding the path without the `@`. A
reference that does not resolve in the finished model is an error
reported at the field: ``unresolved reference `@claims/cl-9` ``.
Resolution happens across all the blocks of a file; across files it
follows the multi-file model of spec section 5.4 when that exists.
Generated code types a reference `String`; the canonical form is as
written; the highlighter shows it distinctly.

### Comments

`//` starts a comment that runs to the end of the line, inside cola
blocks only; prose outside fences is never a comment. A comment is
layout to the grammar and is never a value, so `url: "http://x"` is
unaffected. The marker is `//` because identifiers may contain `-`,
so `--` would need a rule for `a--b`, and because `#` reads as a
heading to anyone looking at the raw markdown.

Comments survive formatting. The model carries them: a comment on its
own line attaches as a **leading** comment to the next field or entity
in the block, or to the close of the enclosing entity when it stands
before `;`, or to the end of the block; a comment after a field, an
entity header or a `;` on the same line is that item's **trailing**
comment. The canonical layout writes leading comments on their own
lines at the item's indentation and a trailing comment after the item
(after its comma) separated by one space; an entity with any comment
in it is never written in the one-line form. Comment text is kept as
written after the marker.

### Strings

Both quote forms decode the same escapes: `\"`, `\'`, `\\`, `\n`,
`\r`, `\t`, and `\u{XXXX}` with one to six hex digits. Any other
escape is an error: ``unknown escape `\q` ``. A string's value is the
decoded text.

A **block string** is `"""` to `"""` and holds prose: no escapes are
decoded, a `"` inside it is literal, and a `"""` cannot be. Its value
is the text with the first line dropped when it holds only whitespace,
the whitespace after the last newline dropped, and the longest common
leading whitespace of the remaining lines removed.

Canonical form: a string containing a newline is written as a block
string, its lines one level deeper than the field and the closing
`"""` at the field's indentation; any other string is double-quoted
with `\\`, `\"`, `\n`, `\r`, `\t` and control characters re-escaped,
except that a string containing `"` and no `'` is single-quoted.

This changes the meaning of existing files that contain a backslash
inside a string: the two characters `\"` were two characters and are
now one. The downstream linter forbade backslashes, and the
conformance corpus has one such file, which the migration note covers.

### Numbers

`[+-]?[0-9][0-9_]*(\.[0-9][0-9_]*)?([eE][+-]?[0-9]+)?`: digit
separators as in Rust, an exponent part. A literal with a decimal
point or an exponent is a float; otherwise an integer, and out of the
`i64` range is an error as today. No leading `.`. Canonical form drops
separators; a float is written in exponent form when its plain decimal
form is longer than 20 characters, so `1e300` does not become three
hundred digits.

### Trailing commas

A comma may follow the last field of a field list, whether `;`, a
nested entity or the block's end comes next, and the last element of a
list. The grammar stays LR(1) by letting a field list end in `,` or in
`, Entity`.

The point of the trailing comma is that appending a field or an entry
never touches the line before it, and the canonical layout keeps that
property: in the multi-line form every field line ends in `,`, the
last included, as does every element line of a multi-line list. The
one-line form, `name: a: 1, b: "x";`, has no comma before the `;`:
nothing is appended to a one-liner without reflowing it, and `,;`
reads wrong.

### Deferred

- **Typed date and instant literals**: unquoted `2026-10-29` lexes as
  numbers, and the schema task can type a string field as an instant.
- **Nulls**: a value that is absent is a field that is omitted.
- **Keeping a decimal as text**: a schema concern.

## Consequences

- Every addition is additive except escape decoding, which is the
  stated migration. Existing conformance files stay valid.
- `ConfigValue` grows `Symbol`, `List` and `Reference`; every match
  on it in the generator, the canonical layout and the highlighter's
  parity test changes. The downstream migration plans for 0.3.0 gain
  these variants.
- The highlighter gains comment, symbol and reference tokens and a
  line state for an open block string.
- The spec's sections 3, 4, 5.5 and 7 are rewritten and section 9
  loses the value-language item; the README's comparison table says
  yes to trailing commas and gains comments.
- Comments in the model make the builder two-pass over a block: parse,
  then scan the block text for `//` outside strings and attach by
  line. That is the one piece of work that does not fall out of the
  grammar, and it is split into its own ticket so the value forms can
  land first.
- The generated plural wrapper and the sample configs under
  `tests/data/` are regenerated where types change.
