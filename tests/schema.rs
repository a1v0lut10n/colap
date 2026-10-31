// SPDX-License-Identifier: Apache-2.0
//! The schema API: loading, validating through a report, and the
//! shapes a loaded schema exposes.
use colap::schema::{Type, Unknown};
use std::path::Path;

#[test]
fn a_loaded_schema_exposes_its_declarations() {
    let loaded = colap::load_schema_file("tests/conformance/schema/registry.schema.md");
    assert!(loaded.is_clean(), "{:?}", loaded.findings);
    let schema = loaded.schema.expect("schema");
    assert_eq!(schema.name.as_deref(), Some("registry"));
    assert_eq!(schema.version, Some(1));
    assert!(schema.utc_suffix);
    let root = schema.shape(schema.root);
    let names: Vec<&str> = root.entities.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["llm", "vendor", "database"]);
    let llm = &root.entities[0];
    assert_eq!(llm.plural.as_deref(), Some("llms"));
    assert!(llm.required);
    assert_eq!(llm.unknown, Unknown::Error);
    assert_eq!(llm.doc.as_deref(), Some("One entry per model."));
    let shape = schema.shape(llm.shape);
    let basis = shape
        .fields
        .iter()
        .find(|f| f.name == "basis")
        .expect("basis");
    assert_eq!(basis.ty, Type::Symbol);
    assert_eq!(basis.one_of.as_ref().map(Vec::len), Some(3));
    let tags = shape
        .fields
        .iter()
        .find(|f| f.name == "tags")
        .expect("tags");
    assert!(tags.list);
    let vendor = shape
        .fields
        .iter()
        .find(|f| f.name == "vendor")
        .expect("vendor");
    assert_eq!(vendor.ty, Type::Reference);
    assert_eq!(vendor.to.as_deref(), Some("vendor"));
    assert_eq!(root.entities[1].unknown, Unknown::Warn);
}

#[test]
fn a_report_validates_and_counts_schema_errors() {
    let schema = colap::load_schema_file("tests/conformance/schema/registry.schema.md")
        .schema
        .expect("schema");
    let mut report = colap::check_file("tests/conformance/schema/registry-findings.md");
    assert!(report.is_clean(), "the file itself is fine");
    assert!(report.findings.is_empty());
    report.validate(&schema);
    assert!(!report.is_clean());
    assert!(report.findings.len() > 20, "{}", report.findings.len());
    // The same findings, without a report.
    let model = colap::parse_file("tests/conformance/schema/registry-findings.md")
        .expect("parses")
        .model;
    let direct = schema.validate(
        &model,
        Some(Path::new("tests/conformance/schema/registry-findings.md")),
    );
    assert_eq!(direct, report.findings);
}

#[test]
fn a_schema_that_does_not_read_reports_why() {
    let loaded = colap::load_schema_file("tests/conformance/schema/missing.schema.md");
    assert!(loaded.schema.is_none());
    assert_eq!(loaded.findings.len(), 1);
    assert!(
        loaded.findings[0]
            .to_string()
            .starts_with("tests/conformance/schema/missing.schema.md: error: "),
        "{}",
        loaded.findings[0]
    );

    // A document that is not a schema at all.
    let loaded = colap::load_schema_file("tests/conformance/valid/values.md");
    assert!(loaded.schema.is_none());
    let text: Vec<String> = loaded.findings.iter().map(|d| d.to_string()).collect();
    assert!(
        text.iter()
            .any(|t| t.contains("missing required entity `schema` in the root")),
        "{text:?}"
    );
}

#[test]
fn the_meta_schema_is_public_and_validates_itself() {
    let parsed = colap::parse_literate_str(colap::schema::META, None).expect("meta parses");
    let loaded = colap::load_schema(&parsed.model, None);
    assert!(loaded.is_clean(), "{:?}", loaded.findings);
    let findings = loaded.schema.expect("meta").validate(&parsed.model, None);
    assert!(findings.is_empty(), "{findings:?}");
}
