---
date: 2026-11-03
type: task
ticket: COLA-0015
status: done
affects:
  - Cargo.toml
  - src/lib.rs
  - src/generator/generator_impl.rs
  - src/generator/schema_gen.rs
  - tests/generate_schema.rs
  - .github/workflows/ci.yml
  - README.md
  - CHANGELOG.md
  - docs/tasks/2026-11/2026-11-01-feature-gate-cli-and-generator-dependencies.md
---

# COLA-0015: the CLI and the generator behind cargo features

Executes `docs/tasks/2026-11/2026-11-01-feature-gate-cli-and-generator-dependencies.md`:
a crate that uses colap only as a library builds only what the library
needs. Follows COLA-0014 (rustemo 0.9.1), since both edit the
manifest and both ship in 0.5.0.

## Inventory (deliverable 1)

| Dependency | Used by | Goes to |
|---|---|---|
| rustemo | `lib.rs`, `format.rs`, the parser | library |
| regex | `schema/dialect.rs` (patterns) | library |
| chrono | `schema/temporal.rs` (`NaiveDate`); also generator headers | library |
| handlebars, heck, serde_json, anyhow | `generator/generator_impl.rs`, `generator/schema_gen.rs` | `generator` |
| clap, env_logger, log | `main.rs` | `cli` |

The task guessed chrono might be generator-only; it is the temporal
types' calendar, so it stays unconditional. `serde_json` is generator-only.

## Action items

- **A1** — `Cargo.toml`: `generator = ["dep:handlebars", "dep:heck",
  "dep:serde_json", "dep:anyhow"]`, `cli = ["generator", "dep:clap",
  "dep:env_logger", "dep:log"]`, `default = ["cli", "generator"]`; the
  seven crates become optional; `[[bin]] colap` gets
  `required-features = ["cli"]`, as do the three tests that drive the
  binary (`cli`, `fmt`, `generate_schema`).
- **A2** — `src/lib.rs`: `#[cfg(feature = "generator")] pub mod generator;`
  and a crate-doc paragraph on the features.
- **A3** — A generated crate depends on the library alone: both
  generators write `colap = { version = "…", default-features = false }`
  (or `{ path = "…", default-features = false }` with `--colap-path`).
- **A4** — CI job `library (no default features)`: clippy over all
  targets, the test suite, and a `cargo tree` check that none of the
  seven gated crates reaches the normal tree (deliverables 3 and 5).
- **A5** — README's Installation section shows `default-features = false`
  and a table of what each feature adds; CHANGELOG under Unreleased.

## What was built

A1 to A5 as written. The library's normal dependency tree goes from 80
crates to 36. `clap`'s `derive` feature is kept as it was, untouched by
this change. Nothing in the library referenced the generator module, so
the gate is one attribute.

## Verification

- [x] `cargo clippy --all-targets --no-default-features -- -D warnings`
      and `cargo test --no-default-features` green.
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test` green with default features; the generated registry
      crate builds against the checkout with `default-features = false`.
- [x] `cargo tree -e normal --no-default-features` lists none of the
      gated crates.
