# Changelog

All notable changes to colap. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Comments.** `//` to the end of the line inside cola blocks, layout to
  the grammar and never a value. The model keeps them (`Comments` on
  every `ConfigField`, and `comments` and `closing` on every
  `EntityNode`; the root's `closing.leading` holds a block's end
  comments) and `colap fmt` writes them back in place; the highlighter
  has `TokenKind::Comment`. Spec sections 3, 5.6 and 7.

### Breaking

- `ModelBuilder::add_block` takes the block text as well as the AST:
  `add_block(&ast, body, line_offset)`. `ConfigField` and `EntityNode`
  gained public fields, and `TokenKind` a variant.

## [0.3.0] — 2026-10-30

The field test of 0.2 in a literate knowledge repository drove this
release: cola is now specified (`docs/spec/cola.md`), checked, formatted,
and held to a conformance corpus. The language is a superset of 0.2
except for string escapes, which are now decoded.

### Breaking

- **Markdown left the grammar.** `ColaParser::parse` parses one cola
  block body; `colap::literate::scan` finds the fences. Read files
  through the facade: `check_file`, `check_str`, `check_literate_str`,
  `check_cola_str` return a `Report` (model or every error, plus
  warnings); the `parse_*` forms return `Result<Parsed, Error>`. A
  markdown file with no cola fence is read as an empty model with a
  warning; a non-markdown file is literate when it contains a cola
  fence and bare cola otherwise.
- **`ModelBuilder` is a struct**: `new(path)`, `add_block(&ast,
  line_offset)`, `record(error)`, `finish()` (resolves references) or
  `finish_block()`. `build_config_model` returns `Result<ConfigModel,
  Error>`.
- **The model is an arena of `EntityNode` only.** `ConfigNode` is gone;
  `get_node` returns `Rc<RefCell<EntityNode>>`; an entity's fields are an
  ordered `Vec<ConfigField>` read through `field`, `field_value` and
  `child`. `ConfigModel::add_entity` and `add_field` enforce the
  duplicate rules of `docs/design/2026-10-29-duplicate-key-semantics.md`:
  a key under a plural is defined once, a slot reopens only with the
  same declaration, a field is set once, and one namespace holds an
  entity's fields and children. What 0.2 silently merged or shadowed is
  now a `ModelError::Duplicate`.
- **`ConfigValue` has new variants** `Symbol`, `Reference` and `List`;
  exhaustive matches must add them. `ConfigValue::String` holds the
  decoded text: `"say \"hi\""` is now `say "hi"`, and an escape outside
  `\" \' \\ \n \r \t \u{…}` is an error.
- **Errors are typed.** `colap::Error` (`Parse`, `Model`, `Io`,
  `Multiple`) displays as `path:line:col: message` with lines and
  columns in the original file; `Error::each()` walks a `Multiple`.
- **The highlighter's `LineState::Cola` and `Code` carry the `Fence`**
  that opened the block; `LineState::BARE_COLA` starts a fence-less
  buffer, `is_cola()` tests the state, and `LineState::ColaString` is a
  line inside a `"""` block string. `TokenKind` gains `Symbol` and
  `Reference`.
- **Generated crates no longer depend on rustemo.** The plural wrapper
  keeps entries in source order (`Vec<(String, T)>` with an index) and
  offers `get`, `keys`, `values`, `iter`, `insert`, `count`; the source
  configuration is copied as `config.<ext>`.
- **The CLI has subcommands.** `colap check FILE... [--format text|cola]`,
  `colap fmt FILE... [--check]` and `colap generate` (the default, so
  `colap FILE` still generates). Exit codes: 0 clean, 1 findings, 2
  usage. `--trace` is global and debug-build only.

### Added

- CommonMark fences: tildes, four or more backticks, indented openers,
  info strings after the language, foreign fenced blocks skipped, any
  prose line accepted, and an unterminated fence reported as a warning
  naming its opening line.
- `colap check`: every error in a file, not the first; warnings for no
  cola blocks, unterminated fences and empty entities; `--format cola`
  renders the findings as a cola block. `Diagnostic` is the flat shape
  for editors and linters.
- `colap fmt`: the canonical layout of `docs/spec/cola.md` section 7,
  applied block by block so prose and foreign fences never move;
  `--check` reports non-canonical blocks.
- The value language of `docs/design/2026-10-30-value-language.md`: bare
  symbols (`basis: parsed`), lists of scalars (`tags: [a, "b", 3]`),
  `@path` references that `check` resolves across the blocks of a file,
  `"""` block strings with common indentation removed, digit separators
  and exponents in numbers, and trailing commas, which the canonical
  multi-line form keeps so an append never touches the line before it.
- The spec, `docs/spec/cola.md`, itself a clean literate document, and
  the conformance corpus under `tests/conformance/` with expected
  diagnostics as sidecars; a test holds the editor highlighter's tokens
  to the parser's terminals.
- `benches/scale.rs`, run in CI: reading stays linear in the number of
  entries (10k entries with a nested entity each in 0.4 s).
- `rustemo` is re-exported for callers that want the raw AST.

### Fixed

- Building a plural with a nested entity under each entry was quadratic.
- An integer past 64 bits is reported as `integer out of range: N` at
  its line and column.
- The generator's templates were HTML-escaped, which turned `Vec<String>`
  into `Vec&lt;String&gt;`; the nested-entity template rendered its
  import path empty inside `each` blocks.
- Lines starting with `#tag` or `#1` are no longer styled as headings.

## [0.2.0] — 2026-08-25

The release the field test ran against: the markdown-aware grammar, the
code generator, and the editor highlighter.

[Unreleased]: https://github.com/a1v0lut10n/colap/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/a1v0lut10n/colap/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/a1v0lut10n/colap/releases/tag/v0.2.0
