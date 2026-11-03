---
date: 2026-11-03
type: journal
components: []
aspects: []
tags: [library, dependencies, features, consumer]
---

# The CLI and the generator behind cargo features (COLA-0015)

## Context

The second finding of the first library consumer: adding
`colap = "0.4"` to a crate added about 200 lines to its lockfile,
because the CLI's and the generator's dependencies were unconditional.
The task of 2026-11-01 proposed `cli` and `generator` features, on by
default, with a `--no-default-features` library build held in CI.

## Details

The inventory settled one open question in the task: `chrono` is the
library's, not the generator's, since the temporal types' calendar
check lives in `schema/temporal.rs`; `serde_json` is the generator's.
So the library proper needs `rustemo`, `regex` and `chrono`, and the
other seven crates became optional behind two features: `generator`
(handlebars, heck, serde_json, anyhow) and `cli` (clap, env_logger,
log, implying `generator`). Both are default, so `cargo install colap`
and a plain dependency line are unchanged. The binary and the three
tests that drive it carry `required-features`; the generator module
is behind one `cfg` attribute, nothing else in the library named it.

The library's normal dependency tree goes from 80 crates to 36. A new
CI job builds, lints and tests the library with the features off and
checks with `cargo tree` that none of the seven gated crates reaches
it. A crate written by `colap generate` now depends on the library
alone, `default-features = false` in its manifest, whether on the
published version or on a checkout.

Follows COLA-0014; both ship in 0.5.0, whose README now shows the
feature table. Replacing handlebars stays with the phenotyper
incubation note, which this change makes local to one feature.

## Links

- Task: `docs/tasks/2026-11/2026-11-01-feature-gate-cli-and-generator-dependencies.md`
- Plan: `docs/implementation/2026-11/2026-11-03-cola-0015-feature-gates.md`
- Planning entry: `docs/journal/2026-11/2026-11-01-plan-library-consumer-fixes.md`
- `docs/incubation/2026-10-31-phenotyper-for-code-generation.md`
