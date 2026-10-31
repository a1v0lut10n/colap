---
date: 2026-10-31
type: task
ticket: COLA-0011
status: in-progress
affects:
  - src/lib.rs
  - src/main.rs
  - src/diagnostic.rs
  - docs/spec/cola.md
  - README.md
  - CHANGELOG.md
  - Cargo.toml
tasks:
  - docs/tasks/2026-10/2026-10-29-schema-validation.md
---

# COLA-0011: the schema validator, `colap check --schema`

## Premise

`docs/design/2026-10-31-schema-dialect.md` decides the dialect, the
checks and the binding; `docs/design/2026-10-31-exact-numbers.md`
(COLA-0010, merged) gives the exact values that `min`, `max` and
`one_of` compare. This ticket builds the validator and its spec. The
generator's `--schema` is the ticket after it.

## Decisions taken in planning

- **`like` joins the dialect.** The meta-schema has to declare a
  declaration, and a declaration contains declarations: the mirrored
  tree cannot say that without naming another declaration. An entity
  declaration may carry `like: @path`, a cola reference to another
  declaration in the same schema, and then has that declaration's
  fields and children (its own `plural`, `required`, `key_pattern`
  and `doc` still apply, and it may declare no `fields` or `entities`
  of its own). The parser resolves the reference; recursion through
  `like` is bounded by the document's depth. Shared shapes such as a
  `provenance` entity under several entries get it for free;
  `extends`, adding to a shape, stays deferred. The design record is
  amended.
- **Findings are `Diagnostic`s.** A validation finding is not an
  `Error` of the language: `Report` gains `findings: Vec<Diagnostic>`,
  filled by `Report::validate(&Schema)`, counted by `is_clean` when of
  error severity and merged by `diagnostics()`. Library users compose
  `check_file` and `validate`; the CLI does the same.
- **Loading a schema is three steps**: read it as any document; check
  its model against the meta-schema with the validator itself; then
  build the `Schema` with the checks no meta-schema can state: `one_of`
  literals of the field's type, `min` not above `max`, `pattern` and
  `key_pattern` compiling, `to` naming a path, `like` pointing at a
  declaration, and the `_utc` naming rule. `load_schema_file(path)`
  returns the schema or every finding, warnings included.
- **The meta-schema is a literate document in the crate**,
  `src/schema/meta.md`, read with `include_str!` and built once; a
  test validates it against itself.
- **Temporal values** are parsed by hand except instants, which
  `chrono` already reads: `date` is `YYYY`, `YYYY-MM`, `YYYY-MM-DD`
  with a real calendar day; `datetime` is a `date` at day precision,
  `T`, `hh:mm`, optional `:ss` and fraction, optional `[Area/Location]`
  checked for shape only; `edtf` is level 0 and level 1: dates with
  `?`, `~`, `%` qualifiers, `X` for unspecified digits, `Y`-prefixed
  long years, seasons `21`–`24`, intervals with `/` and `..` or empty
  open ends. A table in the spec lists exactly what is accepted.
- **The corpus grows a `schema` directory**: `tests/conformance/
  schema/NAME.schema.md` is a schema and every `NAME-*.md` beside it a
  document validated against it, each with its `.expected` sidecar;
  `invalid-*.schema.md` are schemas that fail to load. The runner
  gains a third test.
- **Not here**: `generate --schema`, frontmatter binding, merging
  files before validation.

## Action items

1. **Dialect types** (`src/schema/dialect.rs`): `Schema`,
   `EntityDecl`, `FieldDecl`, `Type`, `Unknown`; built from a
   `ConfigModel` known to match the meta-schema.
2. **Meta-schema** (`src/schema/meta.md`) declaring `schema`, the
   entity and field declarations with `like` for the recursion, and
   every field's type and `one_of`.
3. **Loader** (`src/schema/mod.rs`): `load_schema_file`,
   `load_schema(&ConfigModel, path)`, the semantic checks above,
   `regex` as a direct dependency.
4. **Validator** (`src/schema/validate.rs`): the walk and the nine
   findings of the record with their messages; `Report::validate`.
5. **Temporal** (`src/schema/temporal.rs`): the four parsers and the
   `_utc` offset warning, with unit tests at the edges (February 29,
   `24:00`, `191X`, `1914/..`, `[Europe/Zurich]`).
6. **CLI**: `check --schema S.md FILE...`; the schema's own findings
   printed first; exit 1 on any error; `--format cola` unchanged.
7. **Spec**: `docs/spec/schema.md`, a literate spec with the
   declarations, the types, the attachment of findings, the
   meta-schema by reference and the corpus; `cola.md` section 9 loses
   its last item and section 8 points at schemas; README gains a
   schemas section and the CLI line; CHANGELOG `Unreleased`.
8. **Tests**: the corpus pairs (one schema exercising every
   declaration, documents for every finding, a `like`-recursive tree,
   the temporal types, `utc_suffix: false`), the meta-schema
   self-check, a CLI test, and the task file set done.

## Verification

- [ ] The three Rust gates are clean.
- [ ] Every corpus document's findings equal its sidecar; every
      invalid schema's load findings equal its sidecar; the meta-schema
      validates itself; both spec documents are clean.
- [ ] `colap check --schema` on the knowledge-repository-shaped sample
      reports each kind of violation once, at its line.
- [ ] `cargo bench --bench scale` does not move (validation is not
      on the bench's path).
