---
date: 2026-10-29
type: implementation
ticket: COLA-0001
status: in-progress
tasks:
  - docs/tasks/2026-10/2026-10-29-parse-non-cola-fenced-blocks.md
  - docs/tasks/2026-10/2026-10-29-accept-any-prose-line.md
---

# COLA-0001: a fence-aware scanner in front of a cola-only grammar

## Premise

colap's LR grammar models the whole markdown file: headings, paragraphs,
foreign fences and cola fences are all terminals. That is the root of two
field-test defects (foreign fences never close because the closing line
ties with a code line; prose lines starting with a backtick or a bare `#`
match no terminal) and of several limits (tilde fences, long fences, info
strings, language tags with digits or punctuation).

Markdown is a line-oriented container format, and CommonMark's fence rules
are a dozen lines of scanner. The durable fix the tasks prefer is to take
markdown out of the grammar entirely: a scanner finds fenced blocks, hands
the bodies of the `cola` ones to a grammar that parses cola alone, and the
builder maps every location back to the original file. Prose then needs no
terminals at all, so the prose-line task falls out of the same change.

The parser's public entry changes shape (the grammar's root is no longer
the markdown document), so this is a breaking change for 0.3.0. The
facade it introduces (`colap::parse_file` and friends) is the one the
incubation note `docs/incubation/2026-09-21-standalone-generated-parsers.md`
asks for, and it removes the `rustemo` dependency from generated crates.

## Action items

1. **Scanner** `src/literate.rs`: `scan(text) -> Scan { blocks, warnings }`
   following CommonMark fence rules: backtick or tilde runs of length 3 or
   more, up to 3 spaces of indentation, any info string (first word is the
   language; backtick fences reject backticks in the info string), a closer
   of the same character at least as long with nothing but blanks after it.
   Bodies are kept verbatim with their 1-based start line. An unterminated
   fence runs to the end of the file and yields a warning naming the
   opening line.
2. **Grammar**: `src/grammar/cola.rustemo` loses every markdown rule; the
   root `Cola` is `Entity*`. `build.rs` always refreshes the `src/cola.rustemo`
   copy from the canonical grammar so an edit cannot silently fail to take
   effect.
3. **Builder**: `ModelBuilder` gains an incremental form (`new`, `add_block`
   with a line offset and file path, `finish`) and one location-conversion
   helper; `build_config_model` stays for a single bare block.
4. **Error type** `src/error.rs`: parse errors carry the file path and the
   1-based line and column in the original file; model and I/O errors are
   the other variants. `Display` is `path:line:col: message`.
5. **Facade** in `src/lib.rs`: `parse_literate_str`, `parse_cola_str`,
   `parse_str` (literate if any cola fence is present, bare otherwise) and
   `parse_file` (`.md`/`.markdown` are literate; anything else is detected).
   Each returns the model and the scanner's warnings.
6. **Highlighter** `src/highlight.rs`: `LineState` carries the opening
   fence (`Cola(Fence)`, `Code(Fence)`), so tilde and long fences, info
   strings and the closer-length rule match the scanner. Prose `#` lines are
   headings only when CommonMark says so. (aicogito compares
   `state == LineState::Cola` today and will need a `matches!`.)
7. **CLI and templates**: `main.rs` uses `parse_file` and prints warnings to
   stderr; the generated crate's `Cargo.toml`, README and integration test
   use the facade and drop `rustemo`; the copied test config keeps its
   extension so the dialect detection sees it.
8. **Tests** `tests/literate.rs` with fixtures under `tests/data/literate/`:
   every reproduction from both tasks, locations in the original file for
   entities and for errors in a second block, bare `.cola` input, and a
   scanner/highlighter parity check over all fixtures.
9. **README**: the markdown embedding rules actually supported, the
   statement that prose outside fences is unconstrained, the facade in the
   usage section.

## Verification

- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test` are clean.
- [ ] Every reproduction in the two task files parses, and a foreign block
      containing cola-looking text is ignored.
- [ ] An error in the second cola block of a file reports the file's own
      line and the file path.
- [ ] `colap tests/data/test_genite.md` still generates a crate whose tests
      pass without a `rustemo` dependency.
- [ ] Highlighter state agrees with the scanner on every fixture.
- [ ] README updated; journal entry proposed.

## Outcome

(filled in before the change proposal)
