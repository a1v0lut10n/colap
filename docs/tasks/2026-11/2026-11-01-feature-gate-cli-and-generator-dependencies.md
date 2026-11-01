---
date: 2026-11-01
type: task
status: planned
affects:
  - README.md
tags: [library, dependencies, features, consumer]
---

# Feature-gate the CLI and generator dependencies

## Objective

A crate that uses colap only as a library builds only what the library needs: the
fence scanner, parser, model, `check` and `fmt`, and schemas. The CLI and code generator
dependencies move behind cargo features. The `colap` binary keeps working when installed
with its default features.

## Context

Found 2026-11-01 in the same downstream consumer as the quiet-parser task. Adding
`colap = "0.4"` to a library added about 200 lines to its `Cargo.lock`, because these are
unconditional dependencies today:

| Dependency | Used for |
|---|---|
| clap | the CLI |
| handlebars | the code generator's templates |
| heck | the code generator |
| chrono | generator headers, if still used |
| env_logger | the CLI |
| log | the CLI |
| anyhow | the CLI |

The library also needs `rustemo`, `regex` and possibly `serde_json`; check which. The
incubation note `docs/incubation/2026-10-31-phenotyper-for-code-generation.md` already questions handlebars.
Gating the generator first makes that change local to one feature.

## Deliverables

1. **Inventory.** Map each dependency to the modules that use it: library (`literate`,
   `parser`, `model`, `schema`, `format`, `diagnostic`, `highlight`), `generator`, or
   `main.rs`.
2. **Features.**
   - `cli`: clap, env_logger, log, anyhow, and the binary through `required-features`.
   - `generator`: handlebars, heck, and chrono if it is used, plus the `generator` module.
   - `default = ["cli", "generator"]`, so `cargo install colap` and existing users see no
     change.
3. **CI** builds and tests the library with `--no-default-features`, and the conformance
   corpus and schema tests run in that configuration.
4. **Documentation.** The README's library section shows
   `colap = { version = "0.5", default-features = false }`, with a note on what each
   feature adds.
5. **A dependency-tree check,** so that `cargo tree -e normal --no-default-features`
   lists no CLI or generator crates.

## Explicitly out of scope

- Replacing handlebars (see the phenotyper incubation note).

## Completion

Before setting `status: done`:

- a `--no-default-features` build and its test suite are green;
- README.md documents the features;
- the CHANGELOG notes the change under the release that ships it;
- a journal entry links back to this task.
