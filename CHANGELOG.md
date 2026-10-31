# Changelog

All notable changes to colap. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.0] — 2026-10-31

The knowledge repository's needs after 0.3 drove this release: comments
that survive formatting, numbers that survive at all, and schemas. cola
now has `//` comments, keeps every number exactly as written, and can
declare in a schema the shape a document must have, which `colap check
--schema` enforces and `colap generate --schema` turns into Rust types.
The language is a superset of 0.3 except that an integer past 64 bits is
no longer an error.

### Breaking

- **One number.** `ConfigValue::Integer(i64)` and `ConfigValue::Float(f64)`
  are replaced by `ConfigValue::Number(Number)`, the literal's exact
  decimal of any size with its kind kept from the spelling
  (`is_integer()`); read it with `as_i64()`, `as_f64()`, `decimal()` or
  `text()`. Equality and order are numeric. `canonical::float_text` is
  gone; `integer out of range` has left the diagnostics; the generated
  `i64` reader skips a value that does not fit
  (`docs/design/2026-10-31-exact-numbers.md`).
- **`ModelBuilder::add_block` takes the block text** as well as the
  AST, `add_block(&ast, body, line_offset)`, to find the comments the
  parser skips. `ConfigField` gained `comments`, `EntityNode` gained
  `comments` and `closing`, `Report` gained `findings`, and `TokenKind`
  gained `Comment`: struct literals and exhaustive matches need the
  new members.
- **Generated crates depend on the published colap** (`colap =
  "0.4.0"`) instead of the relative path `../colap`; `--colap-path
  <DIR>` restores a path dependency for local builds.
- The canonical layout writes a number in its own spelling, without
  digit separators, a leading `+` or leading integer zeros: `1.10`
  stays `1.10`, `1e3` stays `1e3`; the former exponent form past twenty
  characters is gone. `colap fmt` may reflow files whose numbers it
  used to rewrite.

### Added

- **Comments.** `//` to the end of the line inside cola blocks, layout
  to the grammar and never a value. The model keeps them (`Comments` on
  every field, and on every entity's header and close; the root's
  `closing.leading` holds a block's end comments) and `colap fmt`
  writes them back in place. Spec sections 3, 5.6 and 7.
- **Exact numbers.** `Number`: `text()`, `is_integer()`, `as_i64()`,
  `as_f64()`, `decimal()` for bignum consumers, `From<i64>`,
  `From<f64>`, `Default`; `ConfigValue` is `PartialEq` and `Eq`.
- **Schemas.** A schema is a cola document mirroring the shape of the
  documents it describes (`docs/spec/schema.md`,
  `docs/design/2026-10-31-schema-dialect.md`): entity declarations with
  `plural`, `required`, `unknown`, `key_pattern`, `like` and `doc`;
  field declarations with `type`, `list`, `required`, `one_of`, `min`,
  `max`, `pattern`, `to` and `doc`; types `string`, `integer`, `float`,
  `number`, `decimal`, `boolean`, `symbol`, `reference`, `instant`,
  `date`, `datetime`, `edtf`, `any`; the `_utc` naming rule. `colap
  check --schema S.md FILE...` validates each file; the library has
  `load_schema_file`, `load_schema`, `Schema::validate` and
  `Report::validate`, with findings as `Diagnostic`s on
  `Report::findings`. The meta-schema (`colap::schema::META`) checks
  every schema first and validates itself. `regex` is a direct
  dependency.
- **`colap generate --schema S.md [INPUT]`** types from declarations:
  one struct per declaration, enums for `symbol` fields with `one_of`,
  `Option` for what is not required, `Vec` for lists, `Number` for
  `decimal`; the generated code embeds the schema and `Root::load`
  validates a file before building it. The input is optional,
  validated first when given, and loaded by the crate's test.
- `Diagnostic::at` builds a finding at a location.

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

[Unreleased]: https://github.com/a1v0lut10n/colap/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/a1v0lut10n/colap/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/a1v0lut10n/colap/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/a1v0lut10n/colap/releases/tag/v0.2.0
