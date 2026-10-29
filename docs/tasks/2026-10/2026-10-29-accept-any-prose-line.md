---
date: 2026-10-29
type: task
status: done
affects:
  - README.md
tags: [grammar, markdown, literate, bug]
---

# Accept any markdown prose line outside cola blocks

## Objective

Any markdown line that is not a fence parses as prose, whatever its first character:
backtick, `#` without a following space, `|`, `<`, `>`, `-`, digits or Unicode.

## Context

Found 2026-10-29 in a downstream knowledge repository. Ordinary reflowed prose such as

    deterministically: ids are slugged, so `ab:CD-123` becomes
    `item:ab.cd-123`. Joins on names are never used.

fails on the second line with `Expected one of STOP, ColaCodeStart, HeadingLine,
ParagraphLine, ...`. Authors must rewrap text whenever an inline code span lands at the
start of a line, and a line-wrapping editor reintroduces the failure at random. The downstream
project had to add a lint rule just to catch these lines before colap did.

**Root cause.** The terminal `ParagraphLine: /[^#`\n][^\n]*\n/` excludes a leading
backtick (to keep fences out) and a leading `#` (to keep headings out), and
`HeadingLine: /#{1,6}[ \t]+[^\n]*\n/` requires a space. A line starting with inline code,
`#hashtag`, `#1`, or seven or more hashes therefore matches no terminal.

## Deliverables

1. **Regression tests** for lines beginning with:
   - an inline code span;
   - a double-backtick span;
   - `#tag`, `#1` and seven hashes;
   - a table pipe, a blockquote and HTML.
2. **The fix.** Preferably fall out of the fence-aware scanner in
   `2026-10-29-parse-non-cola-fenced-blocks.md`. Outside fences, every line is prose,
   and headings need no special terminal because colap does not use them.

   If a grammar-only quick fix ships first, `ParagraphLine` must accept a backtick that is
   not the start of a three-backtick fence, and any `#` line that is not a heading. A
   `#`-led non-heading line must not become an error.
3. **`src/highlight.rs`** stays consistent.

## Explicitly out of scope

- Giving headings or paragraphs semantic meaning in the model, for example
  heading-scoped namespaces. That is a separate design question if ever wanted.

## Completion

Before setting `status: done`:
- the reproductions parse;
- README.md states that prose outside cola fences is unconstrained;
- a journal entry links back to this task.
