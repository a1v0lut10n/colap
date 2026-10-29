---
date: 2026-10-29
type: task
status: done
affects: []
tags: [performance, model]
---

# Scale model building to literate files with thousands of entries

## Objective

Parsing and building a model from a file with 10,000 plural entries, each with about 15
fields, is fast and linear. A benchmark guards it.

## Context

- **Literate knowledge files are large.** A downstream knowledge repository's ledger batches hold hundreds to thousands of
  claim entries per file, and its registry and crosswalk files hold hundreds of entries.
- **Model building is quadratic.** For every nested entity, `ModelBuilder` calls
  `create_entity_at_path`, which calls `find_entity_by_path`. That walks the path from
  the root and scans each level's children linearly by name, so building a plural with n
  entries costs O(n²) children scans.
- **Field nodes double the node count.** Each field is stored both in the entity's map
  and as a separate field node.

## Deliverables

1. **A benchmark** (criterion, or a timed test behind a feature flag) over a generated
   10 k-entry literate file, plus a profile of the current build.
2. **Linear model building.** Build children by parent id rather than by path lookup, and
   keep a per-entity name index. This shares implementation with
   `2026-10-29-define-duplicate-key-semantics.md`.
3. **One field representation,** removing the duplicated map and field-node storage,
   coordinated with `2026-10-29-ordered-model-and-canonical-formatter.md`.
4. **Recorded results.** Benchmark numbers before and after, in the journal entry.

## Explicitly out of scope

- Streaming or incremental parsing.

## Completion

Before setting `status: done`:
- the benchmark runs in CI or is documented as a manual gate;
- a journal entry links back to this task with the numbers.

## Completion note

Done across three tickets: COLA-0002 built entities by parent id with a
per-entity name index, COLA-0004 made the ordered field list the only
field storage, and COLA-0006 added `benches/scale.rs` (run by CI) with
the numbers before and after. The build was quadratic only for entities
nested under keyed entries; a flat ledger was already linear. Parsing
is now 94% of a reading, at about 40 µs per entry.
