---
date: 2026-10-29
type: task
status: planned
affects:
  - README.md
tags: [grammar, markdown, literate, bug]
---

# Parse markdown files that contain non-cola fenced code blocks

## Objective

A markdown file containing any fenced block other than ```` ```cola ```` parses.
Such blocks include ```` ```mermaid ````, ```` ```text ````, ```` ```toml ```` and a bare
```` ``` ````. Their content is skipped, and the surrounding cola blocks still parse.

## Context

Found 2026-10-29 while writing literate cola registries for a downstream knowledge
repository. Every file with a
diagram had to drop it. Minimal reproduction:

    # T

    ```text
    hello
    ```

    Text.

colap 0.2.0 fails at the line after the closing fence with
`Expected one of RegularCodeLine, RegularCodeEnd`. Named and unnamed fences fail alike.

**Root cause (from the grammar, not yet confirmed by a fix).** Inside
`RegularCodeBlock: RegularCodeStart RegularCodeLine* RegularCodeEnd`, the closing line
` ```\n ` matches both `RegularCodeLine: /[^\n]*\n/` and `RegularCodeEnd: /```[ \t]*\n?/`
with the same length. The lexer resolves the tie as a code line, so the block never
closes and the parse runs off the end of the file. The README advertises
"Markdown embedding: first-class", so this is a defect, not a missing feature.

Related limits of the same markdown layer:
- Tilde fences (`~~~`) and fences longer than three backticks (which CommonMark uses to
  nest a ```` ``` ```` example) are not recognised.
- Language tags must be lowercase letters only: `json5`, `c++` and `objective-c` fail.
- Info strings after the tag (```` ```cola title="x" ````) are not accepted.
- An unterminated fence gives a generic parse error rather than "unclosed fence opened
  at line N".

## Deliverables

1. **Regression tests** in `tests/` for each failing shape:
   - named fence, unnamed fence;
   - several foreign blocks interleaved with cola blocks;
   - foreign block containing cola-looking text (must be ignored);
   - fences at end of file without a trailing newline.
2. **Quick fix.** Give `RegularCodeEnd` (and `ColaCodeEnd`) priority over
   `RegularCodeLine`, using rustemo terminal priority, or make `RegularCodeLine` exclude
   lines that open with three backticks. Verify that this actually resolves the tie;
   it is a hypothesis.
3. **Durable fix (preferred).** Move markdown handling out of the LR grammar into a small
   fence-aware line scanner that follows CommonMark fence rules:
   - backtick or tilde fences of length 3 or more;
   - closing fence of the same character and at least the same length;
   - up to 3 spaces of indentation;
   - any info string, whose first word is the language.

   The scanner hands only the cola block bodies, with their line offsets, to a grammar
   that parses cola alone. This also unblocks the fence-less `.cola` file case in
   `docs/incubation/2026-09-21-standalone-generated-parsers.md`, and the prose-line task
   (`2026-10-29-accept-any-prose-line.md`), because prose no longer needs terminals.
4. **Source locations stay correct.** Errors and `SourceLocation`s report line and column
   in the original markdown file, not in the extracted block.
5. **`src/highlight.rs`** recognises the same fence shapes, so editor highlighting
   agrees with the parser.

## Explicitly out of scope

- Interpreting foreign blocks: they are skipped, not validated.
- Duplicate-key semantics across blocks (`2026-10-29-define-duplicate-key-semantics.md`).

## Completion

Before setting `status: done`:
- every reproduction above parses;
- README.md describes the markdown embedding rules actually supported (fence shapes,
  language tags, info strings);
- a journal entry links back to this task.
