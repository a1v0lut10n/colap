---
date: 2026-10-29
type: task
ticket: COLA-0002
status: done
affects:
  - README.md
  - src/model/config_model.rs
design:
  - docs/design/2026-10-29-duplicate-key-semantics.md
tasks:
  - docs/tasks/2026-10/2026-10-29-define-duplicate-key-semantics.md
---

# COLA-0002: enforce the duplicate-key semantics

## Premise

The design record decides what duplicates mean. The model cannot enforce
it as it stands: fields live twice (a map and arena nodes), entities are
attached by path lookup so a repeated sibling captures the next one's
children, and nothing indexes an entity's names. The builder stops at the
first error, while the decision wants every duplicate reported.

## Action items

1. **Model** (`src/model/config_model.rs`): the arena holds entity nodes
   only (`ConfigNode` goes; `get_node` returns the entity). An entity keeps
   an ordered `Vec<ConfigField>` and an ordered `children`, plus a private
   name index covering fields, child slots and keys, and plural names.
   `add_entity(parent_id, …)` applies the key/slot rules and returns the
   existing id on a valid reopen; `add_field` applies the field and
   namespace rules. Both return a `ModelError` whose `Duplicate` carries
   the kind, a message, and both locations. Path lookups use the index.
2. **Errors** (`src/error.rs`): `Error::Model` wraps `ModelError`;
   `Error::Multiple` carries several errors sorted by line; `Display` of a
   located model error is `path:line:col: message; first at line:col`.
3. **Builder**: creates entities by parent id, records every error and
   keeps going (a rejected entity's subtree is skipped), and `finish`
   returns the model or the collected errors. The facade also keeps
   parsing after a block fails, so one bad block no longer hides the next.
4. **Display**: fields print once, in source order, in both renderings.
5. **Generator and templates**: adapt to the flattened arena and
   `EntityNode::field_value`; the plural wrapper's `from_children` reads
   children in order.
6. **Tests** (`tests/duplicates.rs`, fixtures under `tests/data/duplicates/`):
   each rule and each consequence from the design record, including the
   cross-block extension succeeding, the cross-block duplicate key
   failing, the nested-batching case failing, and several errors reported
   at once with exact messages.
7. **README**: a section on reopening and duplicates.

## Verification

- [x] The three Rust gates are clean (53 tests: 17 unit, 12 duplicates,
      12 literate, 11 parser, 1 model builder).
- [x] `tests/duplicates.rs` covers every rule in the design record, with
      exact messages and both locations.
- [x] `format!("{model}")` and `pretty_display` print each field once.
- [x] The genite fixture generates a crate whose three tests pass.
- [x] `aivolution lint .` is clean; README updated; journal entry proposed.

## Outcome

Built as planned. Notes beyond the action items:

- `ModelError` is boxed inside `Error::Model` (clippy's large-error gate);
  `Error::each()` flattens `Multiple` for reporting code.
- The facade keeps parsing after a block fails, so parse errors and model
  errors from one file come back together in line order. That is the
  "recovery at block boundaries" item of the diagnostics task, done early.
- `find_entity_by_path` and `find_child_entity_by_name` accept a plural
  name as well as the singular, since both are in the parent's index.
- Creation by parent id with a per-entity name index makes model building
  linear, which is deliverables 2 and 3 of the scale task; that task keeps
  only its benchmark.

API changes on top of COLA-0001, all pre-0.3.0: `ConfigNode` is gone and
`get_node` returns the entity; `EntityNode.fields` is an ordered
`Vec<ConfigField>` with `field` / `field_value` lookups; `add_entity` and
`add_field` replace `add_child` / `add_field_with_location`;
`ModelBuilder::add_block` records errors and `finish` returns them.
