---
date: 2026-11-01
type: journal
components: []
aspects: []
tags: [library, consumer, rustemo, dependencies]
---

# Plan: two fixes for colap as a library dependency

## Context

colap 0.4 was adopted as a library by a downstream linter for literate cola knowledge
repositories, its first consumer that is neither the CLI nor generated code. Two problems
showed up that neither the CLI nor the conformance corpus exposes.

## Details

Two tasks were pencilled in, both `status: planned`:

- **`docs/tasks/2026-11/2026-11-01-quiet-parser-in-debug-builds.md`.** In a consumer's
  debug build, the parser printed rustemo's trace to stderr: about 3.3 million lines for
  one lint of a few dozen files. `build.rs` sets `notrace(true)`, but the trace comes from
  rustemo's parser at run time. The consumer works around it by building colap and
  rustemo without debug assertions. The task asks for a library-level reproduction, the
  root cause, and a fix that keeps the trace opt-in. Setting the environment from inside
  the library is ruled out.
- **`docs/tasks/2026-11/2026-11-01-feature-gate-cli-and-generator-dependencies.md`.**
  The library pulls in the CLI and generator dependencies (clap, handlebars, heck,
  chrono, env_logger, log, anyhow), about 200 lines in a consumer's lockfile. The task
  proposes `cli` and `generator` features, on by default, with a `--no-default-features`
  library build held in CI.

Suggested order: the quiet parser first, because it affects every consumer's test
output. The features change carries a breaking flag for the next minor release.

## Links

- docs/incubation/2026-10-31-phenotyper-for-code-generation.md, which the generator
  feature would isolate.
