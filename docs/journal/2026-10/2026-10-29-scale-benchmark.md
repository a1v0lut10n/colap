---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0006: the scale benchmark, and what it measured

## Context

The last mechanical task of the literate-format hardening batch: a
benchmark over a 10k-entry literate file guarding the linear model build
that COLA-0002 and COLA-0004 delivered, with numbers before and after.

## Details

The task's premise, that building a plural with n entries was O(n²), was
true only for entities nested under a keyed entry. The old path lookup
resolved the parent, so a flat ledger was found in one step per entry and
was already linear; the pre-COLA-0002 build reads a flat 10k ledger in the
same 0.33 s as today's. The benchmark file therefore carries one nested
entity per entry, the shape of a registry, and on it the old build fails
the linearity check while the current build passes:

| Entries | Old build (1fc470b) | Current build |
|---|---|---|
| 1,000 | 47 ms | 39 ms |
| 10,000 | 1,306 ms | 400 ms |
| 50,000 | 34,253 ms | 1,995 ms |
| Per-entry growth 1k → 50k | 14.6x | 0.99x |

One reading at 10k entries (5.1 MB) today: fence scanning 1%, rustemo
parsing 94%, model building 6%. Parsing costs about 37 µs per entry of
fifteen fields; that is where any further speed-up lives.

`benches/scale.rs` has no dependency: a `harness = false` binary that
generates the ledgers, verifies the models, prints the table and the
breakdown, and exits non-zero when the per-entry cost at 10k exceeds
three times the cost at 1k. CI runs it as a third job. Measured on an
Apple Silicon laptop, release profile, best of five rounds.

## Links

- Task: `docs/tasks/2026-10/2026-10-29-scale-to-large-literate-files.md`
- Plan: `docs/implementation/2026-10/2026-10-29-cola-0006-scale-benchmark.md`
- Earlier: `2026-10-29-duplicate-keys-keys-once-slots-reopen.md` (COLA-0002),
  `2026-10-29-canonical-layout-and-colap-fmt.md` (COLA-0004)
