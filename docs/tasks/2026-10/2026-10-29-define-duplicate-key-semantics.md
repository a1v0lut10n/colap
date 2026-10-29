---
date: 2026-10-29
type: task
status: planned
affects:
  - README.md
tags: [model, semantics, diagnostics, bug]
---

# Define and enforce duplicate-key semantics

## Objective

Duplicate keys have one documented meaning, and the model, the field map, the display
and the generated code all agree on it. Ambiguous duplicates are errors that report both
locations.

## Context

Found 2026-10-29 in a downstream knowledge repository. Its ledger files repeat a
`claim plural claims:` header in several cola blocks of one file, and its linter must
guarantee unique ids. colap 0.2.0 gives three silent, mutually inconsistent behaviours:

1. **Duplicate field in one entity** (`a: 1, a: 2`).
   - `ConfigModel::add_field_to_entity` inserts into `fields: HashMap`, so the last value
     wins.
   - `add_field_with_location` also appends a field node for each occurrence, so the
     node tree holds both.
   - Lookups and the tree therefore disagree.
2. **Duplicate sibling entity** (two `cl-aaa:` entries in one plural).
   - `create_entity_at_path` creates a second node.
   - Nested entities of the second are attached by `find_entity_by_path`, which returns
     the first sibling with that name.
   - The second sibling's fields stay on it, but its nested entities migrate to the first.
3. **The same plural header at the same path in two cola blocks.**
   - A second root entity is created, but its children attach to the first by path
     lookup.
   - The result looks like a merge: entries from both blocks end up under the first
     header, and the second is an empty shell.
   - This is the behaviour that repository relies on, but it happens by accident.

Separately, `ConfigModel`'s `Display` prints each field twice, once from the field map
and once from the field nodes. This makes the tree misleading when debugging exactly
these cases.

## Deliverables

1. **A short design record** (`docs/decisions/ADR-NNN-duplicate-keys.md`) choosing the
   semantics. Recommended:
   - **Fields.** A duplicate field name within one entity is an error.
   - **Plural entries.** A duplicate entry key within the same plural is an error, both
     within one block and across blocks.
   - **Repeated plural headers.** The same header at the same path, with the same
     singular and plural names, *extends* the collection. This makes the literate pattern
     of prose between batches official.
   - **Repeated singular entities.** A repeated singular entity at the same path is an
     error unless an explicit extension form is adopted.
   - **Mismatched plural forms.** The same singular name with a different plural name is
     an error.
2. **The implementation:**
   - create entities by parent id, not by path lookup;
   - index children by name for the duplicate check;
   - return all duplicate errors with both source locations;
   - keep a single source of truth for fields, either the ordered field list or the map,
     but not both.
3. **Fix the double-printing `Display`.**
4. **Tests for every case above,** including the cross-block extension succeeding and the
   cross-block duplicate entry failing.
5. **The generator.** Plural `HashMap` wrappers can no longer silently drop an entry.

## Explicitly out of scope

- Unknown-key validation against a schema (`2026-10-29-schema-validation.md`).

## Completion

Before setting `status: done`:
- the ADR is accepted;
- README.md documents the duplicate and extension rules;
- a journal entry links back to this task.
