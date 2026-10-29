---
date: 2026-10-29
type: task
ticket: COLA-0006
status: done
affects:
  - benches/scale.rs
  - .github/workflows/ci.yml
tasks:
  - docs/tasks/2026-10/2026-10-29-scale-to-large-literate-files.md
---

# COLA-0006: a scale benchmark for large literate files

## Premise

The field-test task asked for four things: a benchmark over a 10k-entry
literate file, a linear model build, one field representation, and the
numbers recorded. The middle two came with COLA-0002 (entities are
created by parent id with a per-entity name index) and COLA-0004 (fields
are one ordered list on the entity). What remains is the benchmark that
guards them, and the measurement that says what was gained.

The gain needs a careful statement. The task's premise was that building
a plural with n entries was O(n²) because every entity was created by a
path lookup that scanned siblings by name. That lookup resolved the
*parent*, so entries directly under a plural were found in one step and
the old build was already linear for a flat ledger. The scan was
quadratic for an entity nested under a keyed entry, whose parent is one
of n siblings. The benchmark file therefore carries one nested entity
per entry, which is also the realistic shape of a registry.

## Action items

1. **The benchmark** `benches/scale.rs`, a plain binary with
   `harness = false` and no new dependency: it generates a ledger batch
   of 1k, 2k, 5k and 10k entries (fifteen fields and one nested entity
   each, blocks of a hundred entries with prose between), reads each
   with `parse_literate_str`, checks the model is complete, prints the
   best of five rounds and the cost per entry, and fails when the cost
   per entry at 10k exceeds three times the cost at 1k. A quadratic
   build is ten times over; a linear one is flat, so the limit is wide
   enough for a shared runner and tight enough to catch the regression.
2. **A breakdown** of one reading at the largest size into fence
   scanning, parsing and model building, printed with the table, so the
   next optimisation starts from where the time goes.
3. **CI** runs the benchmark as a third job, so it stays compiling and
   the linearity check is a gate.
4. **The numbers**, before (the pre-COLA-0002 build at `1fc470b`) and
   after, in the journal entry.

## Verification

- [x] The three Rust gates are clean (clippy covers the bench target).
- [x] `cargo bench --bench scale` passes on the current build with a
      per-entry growth of 1.02x from 1k to 10k entries and 0.99x to
      50k.
- [x] The same bench against the pre-COLA-0002 build fails the
      linearity check (14.6x growth to 50k entries), so the guard
      detects the regression it was written for.
