---
date: 2026-10-30
type: incubation
---

# Temporal values in cola: instants, civil dates with precision, and where they belong

Captured 2026-10-30, from a conversation after the value-language design
(COLA-0007) deferred typed date and instant literals to the schema task.
Hans raised chronology as an important consideration for colap in
knowledge graphs and semantic networks with temporal predicates, to be
discussed before the 0.3.0 release and designed later.

## Two kinds of time, never one type

The ledger and registry files hold two things that look alike and must
not share a type.

- **Transaction time**: when a claim was recorded. A system instant, UTC,
  exact, named `recorded_at_utc` or `created_at_utc`, written
  `2026-10-30T14:05:00Z`. The workspace time conventions apply unchanged.
- **Valid time**: when the asserted fact held in the world ("signed
  1919-06-28", "active from 1914 until 1918", "around 1850"). Civil,
  often imprecise, taken from sources. Forcing it into an instant invents
  a time of day and a zone no source gave. Most temporal predicates in a
  semantic network live here.

A timestamp with `Z` or an offset is an instant; one without is floating
civil time. The difference is semantic and should be visible in the
value, not inferred from the field name. The `_utc` suffix convention
then becomes checkable: a `*_utc` field holding a floating value is a
finding.

## Precision is the real feature

For valid time, a date is an interval at its precision: `1914` the year,
`1914-08` the month, `1914-08-04` the day. Temporal predicates (before,
during, overlaps) are Allen relations between those intervals and only
work when the value carries its precision instead of being padded to
`1914-01-01`. ISO 8601 reduced precision gives the plain forms; EDTF
(Extended Date/Time Format, the Library of Congress extension used by
archives and knowledge bases) adds what sources actually say: `1914?`
uncertain, `~1850` approximate, `191X` unspecified digits, `1914/1918`
intervals, `1914/..` open ends. If the knowledge repository is the main
customer, EDTF level 1 is the specification to adopt rather than
inventing one.

Local civil time with a zone (a meeting at 09:00 in Zurich) is a third
shape, rare in a knowledge graph. When needed, RFC 9557 gives the
spelling `2026-10-30T09:00[Europe/Zurich]`, IANA identifiers only. Zone
abbreviations are never accepted.

## What belongs in the language

The deferral holds for 0.3.0; the work splits in two.

1. **Schema first.** Schema types `instant`, `date`, `datetime` and
   `edtf` applied to string fields, validated by `check`, plus the
   `_utc` naming check. No grammar change; downstream files that already
   write RFC 3339 strings become checked instead of merely conventional.
2. **Literal second, if ordering is needed inside colap.** An unquoted
   temporal literal is lexically clean: `1914-08-04` lexes today as three
   numbers and fails to parse, so a terminal matching the ISO and EDTF
   shapes wins by longest match without touching any existing file. The
   model would gain a `Temporal` value holding kind, precision, offset
   and qualifiers, giving `check` validation (February 30 rejected) and
   the library interval comparison. Worth doing only when something in
   colap itself must order or compare (a schema constraint such as
   `valid_from before valid_until`) or when the highlighter should show
   dates as dates.

Canonical form normalizes spelling but not meaning: an offset stays as
written rather than being converted to `Z`, because the offset is
information about the source. Generated code stays dependency-free with
`String` by default; if typed values are wanted, `jiff` models the civil
versus zoned distinction and RFC 9557 natively, behind a feature flag.
Durations follow the convention as plain numbers with the unit in the
name (`duration_ms`) and need no literal.

## Adjacent work to connect with

- `docs/design/2026-10-30-value-language.md`, which deferred typed date
  and instant literals and nulls to schemas.
- `docs/tasks/2026-10/2026-10-29-schema-validation.md`, the design
  conversation this seeds.
- aivolution-meta's `time-conventions` skill: durable instants are UTC
  with `_utc` names, civil time is explicit with an IANA zone.
- aicogito's `docs/architecture/timestamp-and-clock-conventions.md`, the
  PostgreSQL binding of the same rules.
