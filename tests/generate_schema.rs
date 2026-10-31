// SPDX-License-Identifier: Apache-2.0
//! `colap generate --schema`: the generated crate builds against this
//! checkout and reads typed values; a generated module compiles a
//! `like`-recursive tree; the CLI refuses a document the schema rejects.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn colap(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_colap"))
        .args(args)
        .output()
        .expect("run colap")
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("colap-gen-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn read(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path).expect("read")
}

/// `cargo test` in a generated crate, against this checkout.
fn cargo_test(dir: &Path) -> Output {
    Command::new(env!("CARGO"))
        .args(["test", "-q", "--offline"])
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("run cargo")
}

#[test]
fn the_registry_crate_builds_reads_typed_values_and_rejects_findings() {
    let dir = scratch("registry");
    let out = colap(&[
        "generate",
        "--schema",
        "tests/conformance/schema/registry.schema.md",
        "tests/conformance/schema/registry-clean.md",
        "-o",
        dir.to_str().expect("utf-8"),
        "--colap-path",
        env!("CARGO_MANIFEST_DIR"),
    ]);
    assert_eq!(code(&out), 0, "{}", text(&out.stderr));
    let crate_dir = dir.join("registry-config");
    let lib = read(crate_dir.join("src/lib.rs"));
    for expected in [
        "pub const SCHEMA: &str = r#\"",
        "pub enum LlmBasis {",
        "pub enum ApiType {",
        "pub name: String,",
        "pub max_input_tokens: Option<i64>,",
        "pub output_price: Option<f64>,",
        "pub basis: Option<LlmBasis>,",
        "pub tags: Option<Vec<String>>,",
        "pub weight: Option<colap::config_model::Number>,",
        "pub note: Option<colap::config_model::ConfigValue>,",
        "pub api: Api,",
        "pub extra: Option<Extra>,",
        "pub llms: Llms,",
        "pub database: Option<Database>,",
        "/// One entry per model.",
        "pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, Vec<colap::Diagnostic>>",
    ] {
        assert!(lib.contains(expected), "missing `{expected}` in\n{lib}");
    }
    assert!(read(crate_dir.join("Cargo.toml")).contains(&format!(
        "colap = {{ path = \"{}\" }}",
        env!("CARGO_MANIFEST_DIR")
    )));
    assert!(crate_dir.join("tests/data/config.md").is_file());

    // More tests than the generated one: typed values and a rejection.
    let findings =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/schema/registry-findings.md");
    let extra = format!(
        r#"
#[test]
fn typed_values_come_through() {{
    let root = registry_config::Root::load("tests/data/config.md").expect("valid");
    let gpt = root.llms.get("gpt-4.1").expect("entry");
    assert_eq!(gpt.name, "gpt-4.1");
    assert_eq!(gpt.max_input_tokens, Some(1_048_576));
    assert_eq!(gpt.output_price, Some(8.0));
    assert_eq!(gpt.basis, Some(registry_config::LlmBasis::Parsed));
    assert_eq!(gpt.basis.unwrap().as_str(), "parsed");
    assert_eq!(gpt.tags.as_deref(), Some(&["flagship".to_string(), "vision".to_string()][..]));
    assert_eq!(gpt.vendor.as_deref(), Some("vendors/openai"));
    assert_eq!(gpt.recorded_at_utc, "2026-10-30T14:05:00Z");
    assert_eq!(gpt.weight.as_ref().map(|n| n.text()), Some("0.333333333333333333333333"));
    assert!(gpt.note.is_some());
    assert_eq!(gpt.api.type_, registry_config::ApiType::Rest);
    assert_eq!(gpt.api.key.as_deref(), Some("k"));
    assert!(gpt.extra.is_some());
    let tiny = root.llms.get("tiny").expect("tiny");
    assert_eq!(tiny.basis, None);
    assert_eq!(tiny.api.type_, registry_config::ApiType::GRpc);
    assert_eq!(root.database.as_ref().map(|d| d.url.as_str()), Some("sqlite://registry.db"));
    assert_eq!(root.vendors.get("openai").unwrap().founded.as_deref(), Some("2015-12"));
    assert_eq!(root.llms.keys(), vec!["gpt-4.1", "tiny"]);
}}

#[test]
fn a_document_the_schema_rejects_does_not_load() {{
    let errors = registry_config::Root::load({findings:?}).expect_err("rejected");
    assert!(errors.len() > 20, "{{errors:?}}");
    assert!(errors.iter().all(|e| e.severity == colap::diagnostic::Severity::Error));
}}
"#
    );
    let test_file = crate_dir.join("tests/integration.rs");
    std::fs::write(&test_file, read(&test_file) + &extra).expect("append tests");

    let out = cargo_test(&crate_dir);
    assert_eq!(
        code(&out),
        0,
        "generated crate failed:\n{}\n{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_tree_schema_generates_one_folder_struct_that_contains_its_collection() {
    let dir = scratch("tree");
    let out = colap(&[
        "generate",
        "--schema",
        "tests/conformance/schema/tree.schema.md",
        "-o",
        dir.to_str().expect("utf-8"),
        "-m",
        "module",
        "-n",
        "tree",
    ]);
    assert_eq!(code(&out), 0, "{}", text(&out.stderr));
    let module = read(dir.join("tree.rs"));
    assert_eq!(module.matches("pub struct Folder {").count(), 1);
    assert_eq!(module.matches("pub struct Folders {").count(), 1);
    assert!(module.contains("    pub folders: Folders,"));
    assert!(!module.contains("mod tests"), "no document, no tests");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn generate_refuses_a_document_the_schema_rejects_and_a_broken_schema() {
    let dir = scratch("refused");
    let out = colap(&[
        "generate",
        "--schema",
        "tests/conformance/schema/registry.schema.md",
        "tests/conformance/schema/registry-findings.md",
        "-o",
        dir.to_str().expect("utf-8"),
    ]);
    assert_eq!(code(&out), 1);
    assert!(
        text(&out.stderr).contains("registry-findings.md:14:9: error: unknown field `nmae`"),
        "{}",
        text(&out.stderr)
    );
    assert!(!dir.join("registry-config").exists());

    let out = colap(&[
        "generate",
        "--schema",
        "tests/conformance/schema/invalid-utc.schema.md",
        "-o",
        dir.to_str().expect("utf-8"),
    ]);
    assert_eq!(code(&out), 1);
    assert!(text(&out.stderr).contains("invalid-utc.schema.md:10:17: error:"));

    let out = colap(&["generate", "-o", dir.to_str().expect("utf-8")]);
    assert_eq!(code(&out), 2, "an input is required without a schema");
    let _ = std::fs::remove_dir_all(&dir);
}
