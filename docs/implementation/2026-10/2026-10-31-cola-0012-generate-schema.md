---
date: 2026-10-31
type: task
ticket: COLA-0012
status: done
affects:
  - src/generator/generator_impl.rs
  - src/generator/schema_gen.rs
  - src/generator/mod.rs
  - src/generator/templates/cargo_toml.hbs
  - src/main.rs
  - docs/spec/schema.md
  - docs/design/2026-10-31-schema-dialect.md
  - README.md
  - CHANGELOG.md
tasks:
  - docs/tasks/2026-10/2026-10-29-schema-validation.md
---

# COLA-0012: `colap generate --schema`, types from declarations

## Premise

The schema task's last deliverable: with a schema, the generator types
from declarations instead of guessing from the one literal a sample
holds. `docs/design/2026-10-31-schema-dialect.md` ("Generated code")
says what each declaration becomes; COLA-0011 built the `Schema` the
generator reads.

## Decisions taken in planning

- **A second generator, not a patch on the first.** The inference
  generator walks the sample's model and keeps its oddities (a
  hard-coded `Api` struct, `Models` and `Llms` special cases, two
  templates for one job). With a schema the walk is over
  `Schema::shapes`: one struct per shape, named after its declaration,
  a collection wrapper per plural declaration, a `Root` for the root
  shape. The new path lives in `src/generator/schema_gen.rs` with its
  own templates; the inference path is untouched and stays the
  behaviour without `--schema`.
- **Types.** `string`, `symbol`, `reference`, `instant`, `date`,
  `datetime`, `edtf` are `String`; `integer` is `i64`, `float` and
  `number` are `f64`; `decimal` is `colap::config_model::Number`, the
  exact value, rather than the `String` the record proposed, now that
  the type exists (the record is amended); `boolean` is `bool`; `any`
  is `ConfigValue`. `list: true` wraps in `Vec<T>`; a field that is
  not required is `Option<T>`; a child entity that is not required is
  `Option<Struct>`; a collection is always its wrapper, possibly empty.
  A `symbol` field with `one_of` becomes an enum named after the
  struct and the field (`LlmBasis { Mirrored, Parsed, Derived }`) with
  `as_str`, `FromStr` and `Default` on the first variant; `one_of` on
  any other type stays a constraint. `doc` becomes a doc comment.
- **Names.** A declaration's struct is its name in PascalCase, a
  collection's its plural name; when two declarations would share a
  name, both are prefixed with their parent's struct name. A `like`
  declaration has no struct of its own: its type is the target's, so a
  tree is `Folder { folders: Folders }` of `Folder`.
- **The schema travels with the crate.** The generated crate embeds
  the schema text and gains `Root::load(path) ->
  Result<Root, Vec<colap::Diagnostic>>`, which reads the file, loads
  the embedded schema, validates and builds; `Root::from_model` stays
  for a model the caller has already checked. Reading is lenient as
  today: a value the schema would reject is left at its default.
- **CLI.** `colap generate --schema S.md [INPUT]`: the input is
  optional with a schema; when given it is validated first and
  generation refuses on errors, and it is copied into the crate's
  `tests/data` for the integration test, which goes through `load`.
  The crate name defaults to the schema's `name` field, else the
  schema file's stem, with `-config`. A new `--colap-path` option
  writes a path dependency for local builds; without it the generated
  `Cargo.toml` depends on the published version, which the old
  `../colap` relative path never reliably did (the inference path
  takes the option too).
- **Not here**: typed temporal values (`jiff` behind a feature), a
  `Ref<T>` newtype for references, serde derives.

## Action items

1. **Generator** (`src/generator/schema_gen.rs`): naming pass over the
   shapes, enum and struct emission, collection wrappers, `Root` with
   `from_model` and `load`, the module and crate modes.
2. **Templates**: `schema_enum.hbs`, `schema_struct.hbs`,
   `schema_root.hbs`, `schema_integration_test.hbs`; `plural_struct.hbs`
   reused; `cargo_toml.hbs` takes the dependency line.
3. **CLI**: `--schema` and `--colap-path` on `generate`; the crate
   name default; validation before generation.
4. **Tests**: a crate generated from `registry.schema.md` with
   `registry-clean.md` is built and its integration test run, against
   this checkout through `--colap-path`; a module generated from
   `tree.schema.md` names `Folder` once; the enum, the `Option`s, the
   `Number` and the `like` type are asserted in the generated text.
5. **Docs**: spec `schema.md` section 8 "Generated code" with the type
   table; the record's `decimal` line; README generation section;
   CHANGELOG `Unreleased`; the schema task set done; journal.

## What was built

As planned. Settled on the way:

- `any` is always `Option<ConfigValue>`, required or not:
  `ConfigValue` has no default, and inventing one would read as a
  value.
- The collection wrapper got its own template, a copy of the
  inference path's with `PartialEq`, so that generated structs can
  derive it; the inference path's structs do not, and a shared
  template would have broken them.
- Readers are closures written by the generator and placed by the
  template (`({{read}})(value)`), so the struct template has one arm
  per field shape instead of one per type.
- The generated test for a module mode goes in the module file as
  `mod tests`; without a document no test is generated.
- Enum variants are the symbols in PascalCase (`gRPC` is `GRpc`),
  numbered or prefixed when that collides; `as_str` keeps the symbol.

## Verification

- [x] The three Rust gates are clean.
- [x] `tests/generate_schema.rs` generates the registry crate against
      this checkout, builds it and runs its tests: typed values,
      enums, `Option`s, `Number`, and `Root::load` rejecting
      `registry-findings.md` with the validator's findings.
- [x] A module generated from the tree schema names `Folder` once and
      compiled with its recursion (verified by hand; the test asserts
      the text).
- [x] `cargo bench --bench scale` does not move.
