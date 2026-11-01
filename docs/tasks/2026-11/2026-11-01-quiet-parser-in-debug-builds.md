---
date: 2026-11-01
type: task
status: planned
affects:
  - README.md
tags: [library, diagnostics, rustemo, consumer]
---

# Keep the parser quiet in a consumer's debug builds

## Objective

A crate that depends on colap 0.4 as a library, and builds in its default dev profile,
prints nothing from colap's parser unless it asks for a trace. Today it gets rustemo's
full parse trace on stderr.

## Context

Found 2026-11-01 in a downstream library consumer: a linter that reads a few dozen
literate cola files of a knowledge repository, each a few hundred lines. In the
consumer's dev and test builds, one lint run wrote about 3.3 million lines of trace to
stderr ("Recognizing STOP -- recognized", parse stacks, states). Tests and examples
became slow, and their output was unreadable.

What is known:

- **The trace depends on debug assertions in the consumer's build.** It appears whenever
  colap and rustemo are compiled with debug assertions, which is the default dev and test
  profile of any consumer. The 0.3 changelog says the CLI's `--trace` is global and
  debug-build only, but nothing stops the trace for library callers.
- **colap's own setting does not cover it.** `build.rs` already calls
  `settings.notrace(true)`, and that covers what rustemo generates. The generated
  `src/parser/cola.rs` still imports `rustemo::debug::{log, logn}` (and
  `rustemo::colored` under `cfg(debug_assertions)`), and the trace comes from rustemo's
  LR parser at run time.
- **Two workarounds help today.** One is setting `RUSTEMO_NOTRACE=1` in the environment.
  The other is a profile override in the consumer's workspace:

      [profile.dev.package.colap]
      debug-assertions = false

      [profile.dev.package.rustemo]
      debug-assertions = false

  The consumer adopted the override. It is not yet known whether the rustemo entry alone
  suffices.

## Deliverables

1. **A reproduction test.** A library-level test, built with debug assertions, that
   parses a literate file through `check_file` and captures stderr. It must show no trace
   output.
2. **Root cause.** Pin down which crate's `debug_assertions` gate emits the trace, and
   whether `notrace(true)` can reach it.
3. **A fix that keeps the trace opt-in for library callers,** in order of preference:
   - a parser setting that turns rustemo's runtime logging off for colap's parser;
   - an upstream rustemo option, with colap pinning the release that has it;
   - as a last resort, documented guidance (the profile override above) in the README's
     library section.

   Setting the environment variable from inside the library is not acceptable: it is
   process-global, and not thread-safe.
4. **The CLI keeps its trace.** `colap --trace` still works in debug builds.

## Explicitly out of scope

- Changing rustemo's trace format.

## Completion

Before setting `status: done`:

- the reproduction test passes in a debug build;
- README.md says how a library caller gets a trace if it wants one;
- a journal entry links back to this task.
