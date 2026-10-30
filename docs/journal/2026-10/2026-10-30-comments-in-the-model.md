---
date: 2026-10-30
type: journal
components: []
aspects: []
---

# COLA-0009: comments carried through the model and `fmt`

## Context

The value-language design record accepted `//` comments but COLA-0007
left them out: the parser skips layout, so comments do not fall out of
the grammar and the model has to carry them for `colap fmt` to keep
them. COLA-0009 closes the value-language task.

## Details

The grammar's layout terminal now accepts `//` to the end of the line
beside whitespace, as one regex. A separate `Comment` terminal parsed
correctly but cost 17% on the scale benchmark, because the layout
parser runs between every two tokens; the single terminal costs
nothing measurable against main on the same machine.

The builder finds comments with the highlighter's line lexer, which
already knows strings and block strings across lines, so the parser,
the editor and the formatter agree on what a comment is. One
source-order walk attaches each: on the line where a field, a header
or a `;` ends it trails that item; inside a multi-line list it leads
the field; otherwise it leads the next item of the innermost entity,
or that entity's `;`, or the end of the block. The model holds them as
`Comments { leading, trailing }` on every field and twice on every
entity (header and close); the root's close holds block-end comments.
The canonical layout writes them back in place; an entity with a
comment on its header, a field or before its `;` is never written on
one line, while `a: x: 1; // note` stays one line.

`ModelBuilder::add_block` now takes the block text, the one breaking
change, recorded under Unreleased in the changelog.

## Links

- Plan: `docs/implementation/2026-10/2026-10-30-cola-0009-comments.md`
- Design: `docs/design/2026-10-30-value-language.md`
- Task: `docs/tasks/2026-10/2026-10-29-value-language-extensions.md` (done)
- Spec sections 3, 5.6, 7 and 9 in `docs/spec/cola.md`
