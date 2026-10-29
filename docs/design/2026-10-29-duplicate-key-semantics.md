---
date: 2026-10-29
type: design
status: accepted
components: []
aspects: []
tags: [model, semantics, literate]
---

# Duplicate names: keys are defined once, slots reopen, fields are set once

## Context

colap 0.2 accepted duplicate names silently and inconsistently. A field
repeated in one entity was last-wins in the lookup map but kept twice in
the node tree; a sibling entity repeated under the same parent got a
second node whose nested entities migrated to the first by path lookup;
and the same plural header in two cola blocks looked like a merge — the
behaviour a downstream knowledge repository's append-only ledgers came to
rely on, by accident.

That repository needs two guarantees from the format: a ledger file may
spread one collection over several cola blocks with prose between the
batches, and an entry id may never appear twice. Both must hold across a
file, and later across the files given to one `colap check`.

cola already distinguishes two kinds of name. A name directly under a
`plural` header is data: it identifies an entry and becomes a map key in
generated code. Every other name is structure: it names a place in the
configuration and becomes a struct field. The decision follows that line.

## Decision

A name directly under a plural is a **key**. Any other entity name is a
**slot**. A field name is a **field**.

1. **Keys are defined once.** Two entries with the same key under the same
   plural are an error, whether in one block or in different blocks. The
   second definition is reported with both locations and its subtree is
   not added to the model.
2. **Slots may be reopened.** The same declaration at the same path
   continues the same node: a reopened plural appends entries, a reopened
   singular adds fields and nested slots. The node keeps the location of
   its first opening. Reopening a slot with a different declaration — a
   different plural name, or singular where it was plural and vice versa —
   is an error with both locations.
3. **Fields are set once.** A field name repeated in one entity, across
   reopenings or not, is an error with both locations.
4. **One namespace per entity.** Within an entity, a name is a field or an
   entity, never both, and a plural declaration reserves both its singular
   and its plural name. Clashes are errors with both locations.
5. **All duplicate errors are reported**, not just the first; the model
   keeps every source order; and the model holds each field once.

Equality of names is exact and case-sensitive. "The same path" is the
chain of singular names from the root.

## Consequences

- The literate pattern of prose between batches of one collection is
  official, and so is splitting a singular configuration section across
  blocks by narrative. A multi-file `colap check` can apply the same rules
  in command-line order and detect cross-file duplicate keys.
- **Nested batching under a key is not possible.** Adding models to
  `llm/openai` from a later block means reopening the entry `openai`,
  which rule 1 forbids. Such collections are written in one block, or
  restructured. An explicit continuation syntax remains a possible later
  extension and is not adopted now.
- The override idiom — defaults first, a later block setting a field
  again — is an error rather than last-wins. No file in the workspace
  used it.
- Generated plural wrappers can no longer drop an entry, and the
  one-namespace rule is what generated Rust structs required anyway.
  Names that collide only after Rust sanitising (`a-b` and `a_b`) are a
  generator concern, left to schema-driven generation.
- The model needs a per-entity name index and creation by parent id
  rather than by path lookup, which also makes model building linear in
  the number of entries.
- The error surface grows a duplicate form carrying two locations and a
  multiple-errors form; the diagnostics work builds its reporting
  channel on them.
