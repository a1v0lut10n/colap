---
date: 2026-10-29
type: journal
components: []
aspects: []
---

# COLA-0002: keys are defined once, slots reopen, fields are set once

## Context

colap 0.2 accepted duplicate names silently, with three inconsistent
behaviours: a repeated field was last-wins in the map but kept twice in the
tree; a repeated sibling entity got a second node whose nested entities
migrated to the first by path lookup; and a plural header repeated across
cola blocks looked like a merge by accident. The knowledge repository's
ledgers need exactly two guarantees: batches of one collection across
blocks, and unique entry ids. COLA-0002 decided the semantics and made the
model enforce them.

## Details

The design record draws the line cola already draws with `plural`:

- a name directly under a plural is a **key** and is defined once;
- every other entity name is a **slot** and may be reopened with the same
  declaration, continuing the same node;
- a field is set once, and an entity has one namespace — a name is a field
  or an entity, never both, and a plural declaration reserves both names.

Decided in session: keys are strictly defined once (no continuation form
for now), the node arena is flattened to entity nodes, and the branch
started after PR #18 merged. The consequence worth remembering: nested
batching under a key is impossible, because reaching `llm/openai/models`
from a later block means reopening the key `openai`. Such collections are
written in one block.

Implementation: a private name index per entity, creation by parent id,
`add_entity` returning the existing id on a valid reopen, `ModelError`
with both locations (boxed inside `Error::Model`), `Error::Multiple`
sorted by line, parse recovery at block boundaries so one bad block no
longer hides the next, and fields printed once in both renderings. Model
building is now linear in the number of entries, which closes two
deliverables of the scale task early; that task keeps its benchmark.

Twelve new tests in `tests/duplicates.rs` assert exact messages and both
locations for every rule and every consequence; 53 tests pass.

## Links

- Design record: `docs/design/2026-10-29-duplicate-key-semantics.md`
- Plan: `docs/implementation/2026-10/2026-10-29-cola-0002-duplicate-keys.md`
- Task closed: `docs/tasks/2026-10/2026-10-29-define-duplicate-key-semantics.md`
- Previous entry: `2026-10-29-fence-scanner-replaces-markdown-grammar.md`
