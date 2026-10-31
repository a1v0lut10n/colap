// SPDX-License-Identifier: Apache-2.0
//! Schemas: a cola document declaring the shape other documents must
//! have, and the validator that checks them against it
//! (`docs/design/2026-10-31-schema-dialect.md`, `docs/spec/schema.md`).
//!
//! Loading a schema is three steps: read it as any document, validate
//! its model against the built-in meta-schema with this same
//! validator, then build the [`Schema`] with the checks no meta-schema
//! can state. Validating a document is [`Schema::validate`], or
//! [`crate::Report::validate`] on a report. Findings are
//! [`Diagnostic`]s: the same channel as parse errors and warnings.

mod dialect;
mod temporal;
mod validate;

pub use dialect::{EntityDecl, FieldDecl, Schema, Shape, ShapeId, Type, Unknown};

use crate::diagnostic::{Diagnostic, Severity};
use crate::model::config_model::ConfigModel;
use std::path::Path;
use std::sync::OnceLock;

/// The meta-schema: the dialect declared in the dialect, a literate
/// document.
pub const META: &str = include_str!("meta.md");

/// A schema as loaded: the schema unless an error was found, and
/// every finding either way.
#[derive(Debug)]
pub struct Loaded {
    pub schema: Option<Schema>,
    pub findings: Vec<Diagnostic>,
}

impl Loaded {
    pub fn is_clean(&self) -> bool {
        self.schema.is_some() && !has_errors(&self.findings)
    }
}

pub(crate) fn has_errors(findings: &[Diagnostic]) -> bool {
    findings.iter().any(|d| d.severity == Severity::Error)
}

/// The meta-schema, built once. `None` only if the document shipped
/// in the crate is broken, which a test rules out.
fn meta() -> Option<&'static Schema> {
    static META_SCHEMA: OnceLock<Option<Schema>> = OnceLock::new();
    META_SCHEMA
        .get_or_init(|| {
            let parsed = crate::parse_literate_str(META, Some(Path::new("meta.md"))).ok()?;
            let (schema, findings) = dialect::build(&parsed.model, Some(Path::new("meta.md")));
            (!has_errors(&findings)).then_some(schema).flatten()
        })
        .as_ref()
}

/// Load the schema a model declares: checked against the meta-schema,
/// then built. `path` names the schema in findings.
pub fn load_schema(model: &ConfigModel, path: Option<&Path>) -> Loaded {
    let Some(meta) = meta() else {
        return Loaded {
            schema: None,
            findings: vec![Diagnostic::at(
                Severity::Error,
                path,
                None,
                "the built-in meta-schema failed to load; this is a bug in colap".into(),
            )],
        };
    };
    let mut findings = validate::validate(meta, model, path);
    if has_errors(&findings) {
        return Loaded {
            schema: None,
            findings,
        };
    }
    let (schema, more) = dialect::build(model, path);
    findings.extend(more);
    findings.sort_by_key(|d| (d.line.unwrap_or(0), d.column.unwrap_or(0)));
    Loaded { schema, findings }
}

/// Load a schema file: read like any document, with the reading's own
/// errors and warnings among the findings.
pub fn load_schema_file(path: impl AsRef<Path>) -> Loaded {
    let path = path.as_ref();
    let report = crate::check_file(path);
    match &report.outcome {
        Ok(model) => {
            let mut loaded = load_schema(model, Some(path));
            let mut findings = report.diagnostics();
            findings.append(&mut loaded.findings);
            loaded.findings = findings;
            loaded
        }
        Err(_) => Loaded {
            schema: None,
            findings: report.diagnostics(),
        },
    }
}

impl Schema {
    /// Every finding of a document's model against this schema, in
    /// line order; `path` names the document in findings.
    pub fn validate(&self, model: &ConfigModel, path: Option<&Path>) -> Vec<Diagnostic> {
        validate::validate(self, model, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_meta_schema_loads_and_validates_itself() {
        let parsed = crate::parse_literate_str(META, Some(Path::new("meta.md"))).expect("parses");
        let loaded = load_schema(&parsed.model, Some(Path::new("meta.md")));
        assert!(loaded.findings.is_empty(), "{:?}", loaded.findings);
        let schema = loaded.schema.expect("builds");
        let findings = schema.validate(&parsed.model, Some(Path::new("meta.md")));
        assert!(findings.is_empty(), "{findings:?}");
        // `like` made the nested declaration share the outer one's shape.
        let root = schema.shape(schema.root);
        let decl_schema = &root.entities[0];
        let entity = &schema.shape(decl_schema.shape).entities[0];
        let nested = &schema.shape(entity.shape).entities[1];
        assert_eq!(nested.name, "entity");
        assert_eq!(nested.shape, entity.shape);
    }
}
