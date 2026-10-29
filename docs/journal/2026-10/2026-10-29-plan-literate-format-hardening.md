---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# Plan: harden cola as a literate representation format (nine tasks)

## Context

A downstream project is designing a git-backed, append-only knowledge repository. It
chose cola embedded in markdown, instead of YAML or JSON, as its representation format.

Writing its registry, ledger and crosswalk files against colap 0.2.0 exposed parser
defects and format gaps:

- **Foreign fences.** Any non-cola fenced block fails to parse.
- **Backtick-led prose.** A prose line starting with a backtick fails to parse.
- **Duplicate keys.** They are accepted silently, with inconsistent results.
- **No way to check without generating.** There is no validate-only mode, and errors
  carry no file path.
- **No ordering or determinism.** Field order is not preserved, and nothing writes cola
  back out.
- **No schema.** Nothing declares or checks a file's shape.

All parse findings were reproduced with minimal files. Root causes were traced in
`src/grammar/cola.rustemo`, `src/model/model_builder.rs` and `src/model/config_model.rs`.
The terminal-priority quick fix for fences is recorded as a hypothesis; no colap source
was changed or rebuilt.

## Details

Nine tasks were pencilled in, all `status: planned`.

The three defects found:

- `docs/tasks/2026-10/2026-10-29-parse-non-cola-fenced-blocks.md`: the closing fence
  ties between `RegularCodeLine` and `RegularCodeEnd`. The preferred fix is a
  CommonMark-style fence scanner in front of a cola-only grammar.
- `docs/tasks/2026-10/2026-10-29-accept-any-prose-line.md`: `ParagraphLine` excludes a
  leading backtick or `#`. The fence scanner removes the need for prose terminals.
- `docs/tasks/2026-10/2026-10-29-define-duplicate-key-semantics.md`: the field map
  disagrees with the field nodes; entities are attached by path; repeated plural headers
  merge by accident. The task proposes an ADR: repeated plural headers extend the
  collection, and every other duplicate is an error.

Improvements toward a representation format:

- `docs/tasks/2026-10/2026-10-29-colap-check-and-diagnostics.md`: `colap check`,
  `path:line:col` diagnostics, exit codes 0/1/2, trace off by default.
- `docs/tasks/2026-10/2026-10-29-ordered-model-and-canonical-formatter.md`: preserve
  source order, and add `colap fmt` with a round-trip guarantee.
- `docs/tasks/2026-10/2026-10-29-value-language-extensions.md` (ADR first): symbols,
  scalar lists, references, comments, decoded escapes, multi-line and exponent
  literals, trailing commas.
- `docs/tasks/2026-10/2026-10-29-schema-validation.md`: schemas written in cola, plus
  `colap check --schema`, plus schema-driven code generation.
- `docs/tasks/2026-10/2026-10-29-normative-spec-and-conformance-corpus.md`: a spec, a
  valid/invalid corpus, a single grammar source, highlighter parity.
- `docs/tasks/2026-10/2026-10-29-scale-to-large-literate-files.md`: linear model build,
  with a 10k-entry benchmark.

Suggested order:

1. the fence scanner (which fixes fences and prose lines together);
2. duplicate semantics and linear model building, which share one change;
3. check and diagnostics;
4. order and the formatter;
5. the spec and corpus;
6. the two ADR-led tasks.

## Links

- Related incubation: `docs/incubation/2026-09-21-standalone-generated-parsers.md`. It
  covers the facade, fence-less `.cola` files and the rustemo trace default; these tasks
  reference it rather than repeat it.
