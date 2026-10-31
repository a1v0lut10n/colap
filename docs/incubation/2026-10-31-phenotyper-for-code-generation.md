---
date: 2026-10-31
type: incubation
---

# Phenotyper in place of the handlebars templates

Captured 2026-10-31, after the colap 0.4.0 release and the downstream
migrations, when Hans asked to pencil in a look at what replacing the
generator's handlebars templates with phenotyper would involve.

colap's code generator renders Rust from thirteen handlebars templates
under `src/generator/templates/` (`entity_struct`, `plural_struct`,
`schema_struct`, `schema_enum`, `schema_collection`, the Cargo manifest,
README and integration tests, and the rest). A template engine
substitutes values into a textual skeleton: it knows nothing of what
the output is, so the shape of a generated struct lives half in the
template text and half in the Rust that prepares the context for it,
and nothing can read a generated crate back.

phenotyper is our own DSL and compiler for the shape of structured
textual artifacts: a source file declares types, named phenotypes
with fields and render expressions, singular and plural companions
and nested phenotypes, and the compiler generates typed builders,
renderers and validators for them. One of its ambitions is that the
same definitions drive both generation and parsing of the artifact,
with round-tripping to follow.

The idea is to describe the generated Rust as a phenotyper artifact
family, so that colap builds a typed instance of it from a schema or
a sample model and renders it, rather than filling templates. What
that would involve is the question to look into: whether phenotyper's
render expressions can say what the templates say today (doc
comments, optional and list fields, the enum with `as_str` and
`FromStr`, the collection wrapper, the embedded schema text and the
`load` function), what the generator's context-preparing code in
`schema_gen.rs` and the inference path becomes, where the two
generators' shared skeletons meet, and what the parsing ambition
would give colap once it lands: reading a generated crate back to
check it against the schema that produced it, or to regenerate
without clobbering what a user edited.

## Adjacent work to connect with

- phenotyper itself (`/Users/hansvw/Aivolution/Projects/phenotyper`),
  a related repo in the workspace manifest; its parser is a Rustemo
  GLR grammar, the same parser generator colap uses.
- `docs/incubation/2026-09-21-standalone-generated-parsers.md`, the
  other idea about what the generator emits.
- `docs/spec/schema.md` section 7, the current contract for generated
  code, which any replacement must keep.
