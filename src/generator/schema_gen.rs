// SPDX-License-Identifier: Apache-2.0
//! `colap generate --schema`: Rust types from a schema's declarations
//! (`docs/design/2026-10-31-schema-dialect.md`, "Generated code").
//!
//! The walk is over the schema's shapes, not a sample document: one
//! struct per shape, named after the declaration that owns it, a
//! collection wrapper per plural declaration, an enum per `symbol`
//! field with `one_of`, and a `Root` for the root shape that can
//! `load` a file through the embedded schema.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Local;
use handlebars::Handlebars;
use heck::{ToPascalCase, ToSnakeCase};
use serde_json::{Value, json};

use super::generator_impl::GenerationMode;
use crate::model::config_model::ConfigValue;
use crate::schema::{EntityDecl, FieldDecl, Schema, ShapeId, Type};

const CONFIG_VALUE: &str = "colap::config_model::ConfigValue";

/// Generates a crate or module from a schema.
pub struct SchemaGenerator {
    schema: Schema,
    /// The schema's text, embedded in the generated code for `Root::load`.
    schema_text: String,
    schema_path: PathBuf,
    /// A document that matches the schema, copied in for the tests.
    document: Option<PathBuf>,
    mode: GenerationMode,
    colap_dependency: String,
    handlebars: Handlebars<'static>,
    names: Names,
}

/// What the walk over the shapes emits.
#[derive(Default)]
struct Emitted {
    done: HashSet<ShapeId>,
    structs: Vec<Value>,
    enums: Vec<Value>,
    /// `(wrapper, element struct)` per collection.
    collections: Vec<(String, String)>,
}

/// A child declaration, read out of the schema before the walk
/// borrows `self` mutably.
struct Child {
    name: String,
    plural: Option<String>,
    required: bool,
    doc: Option<String>,
    shape: ShapeId,
}

/// The names the shapes, collections and enums got, one namespace.
#[derive(Default)]
struct Names {
    taken: HashSet<String>,
    struct_of: HashMap<ShapeId, String>,
    /// `(shape, plural name)` to the wrapper's name.
    wrapper_of: HashMap<(ShapeId, String), String>,
}

impl Names {
    fn claim(&mut self, candidates: impl IntoIterator<Item = String>) -> String {
        let mut last = String::new();
        for candidate in candidates {
            if self.taken.insert(candidate.clone()) {
                return candidate;
            }
            last = candidate;
        }
        // Every candidate was taken: number the last one.
        let mut n = 2;
        loop {
            let numbered = format!("{last}{n}");
            if self.taken.insert(numbered.clone()) {
                return numbered;
            }
            n += 1;
        }
    }
}

/// A Rust identifier for a cola name: snake case, keywords suffixed.
fn field_ident(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
        "gen",
    ];
    let snake = name.to_snake_case();
    if KEYWORDS.contains(&snake.as_str()) {
        format!("{snake}_")
    } else {
        snake
    }
}

/// A raw string literal holding `text`, with enough hashes.
fn raw_string(text: &str) -> String {
    let mut hashes = String::new();
    while text.contains(&format!("\"{hashes}")) {
        hashes.push('#');
    }
    format!("r{hashes}\"{text}\"{hashes}")
}

/// The element type and the reader closure for a field's type.
fn scalar_type(decl: &FieldDecl, enum_name: Option<&str>) -> (String, String) {
    let cv = CONFIG_VALUE;
    match (decl.ty, enum_name) {
        (Type::Symbol, Some(name)) => (
            name.to_string(),
            format!(
                "|v: &{cv}| match v {{ {cv}::Symbol(s) => s.parse::<{name}>().ok(), _ => None }}"
            ),
        ),
        (
            Type::String
            | Type::Symbol
            | Type::Reference
            | Type::Instant
            | Type::Date
            | Type::Datetime
            | Type::Edtf,
            _,
        ) => (
            "String".into(),
            format!(
                "|v: &{cv}| match v {{ {cv}::String(s) | {cv}::Symbol(s) | {cv}::Reference(s) => Some(s.clone()), _ => None }}"
            ),
        ),
        (Type::Integer, _) => (
            "i64".into(),
            format!("|v: &{cv}| match v {{ {cv}::Number(n) => n.as_i64(), _ => None }}"),
        ),
        (Type::Float | Type::Number, _) => (
            "f64".into(),
            format!("|v: &{cv}| match v {{ {cv}::Number(n) => Some(n.as_f64()), _ => None }}"),
        ),
        (Type::Decimal, _) => (
            "colap::config_model::Number".into(),
            format!("|v: &{cv}| match v {{ {cv}::Number(n) => Some(n.clone()), _ => None }}"),
        ),
        (Type::Boolean, _) => (
            "bool".into(),
            format!("|v: &{cv}| match v {{ {cv}::Boolean(b) => Some(*b), _ => None }}"),
        ),
        (Type::Any, _) => (cv.to_string(), format!("|v: &{cv}| Some(v.clone())")),
    }
}

impl SchemaGenerator {
    pub fn new(
        schema: Schema,
        schema_text: String,
        schema_path: PathBuf,
        document: Option<PathBuf>,
        mode: GenerationMode,
    ) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("enum", include_str!("templates/schema_enum.hbs"))?;
        handlebars
            .register_template_string("struct", include_str!("templates/schema_struct.hbs"))?;
        handlebars.register_template_string(
            "collection",
            include_str!("templates/schema_collection.hbs"),
        )?;
        handlebars.register_template_string(
            "integration_test",
            include_str!("templates/schema_integration_test.hbs"),
        )?;
        handlebars
            .register_template_string("cargo_toml", include_str!("templates/cargo_toml.hbs"))?;
        handlebars.register_template_string("readme", include_str!("templates/readme.hbs"))?;
        handlebars.register_escape_fn(handlebars::no_escape);
        Ok(Self {
            schema,
            schema_text,
            schema_path,
            document,
            mode,
            colap_dependency: format!("\"{}\"", env!("CARGO_PKG_VERSION")),
            handlebars,
            names: Names::default(),
        })
    }

    /// The `colap = …` line of the generated `Cargo.toml`: a path
    /// dependency for local builds.
    pub fn with_colap_path(mut self, path: &Path) -> Self {
        self.colap_dependency = format!("{{ path = {} }}", raw_toml_string(path));
        self
    }

    pub fn generate(&mut self) -> Result<()> {
        let code = self.code()?;
        match self.mode.clone() {
            GenerationMode::Module { output_file } => {
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out = code;
                if let Some(document) = &self.document {
                    out.push_str(&self.tests(false, &document.to_string_lossy())?);
                }
                fs::write(&output_file, out)
                    .with_context(|| format!("writing {}", output_file.display()))?;
            }
            GenerationMode::Crate {
                output_dir,
                crate_name,
            } => {
                fs::create_dir_all(output_dir.join("src"))?;
                let cargo = self.handlebars.render(
                    "cargo_toml",
                    &json!({ "crate_name": crate_name, "colap_dependency": self.colap_dependency }),
                )?;
                fs::write(output_dir.join("Cargo.toml"), cargo)?;
                fs::write(output_dir.join("src").join("lib.rs"), code)?;
                if let Some(document) = &self.document {
                    let data_dir = output_dir.join("tests").join("data");
                    fs::create_dir_all(&data_dir)?;
                    let name = match document.extension().and_then(|e| e.to_str()) {
                        Some(ext) => format!("config.{ext}"),
                        None => "config".into(),
                    };
                    fs::copy(document, data_dir.join(&name))
                        .with_context(|| format!("copying {}", document.display()))?;
                    let tests = self.tests(true, &format!("tests/data/{name}"))?;
                    fs::write(output_dir.join("tests").join("integration.rs"), tests)?;
                }
                let readme = self.handlebars.render(
                    "readme",
                    &json!({
                        "crate_name": crate_name,
                        "config_filename": self.schema_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                        "date": Local::now().format("%Y-%m-%d").to_string(),
                        "example_code": "",
                        "from_schema": true,
                    }),
                )?;
                fs::write(output_dir.join("README.md"), readme)?;
            }
        }
        Ok(())
    }

    fn crate_name(&self) -> String {
        match &self.mode {
            GenerationMode::Crate { crate_name, .. } => crate_name.replace('-', "_"),
            GenerationMode::Module { .. } => "crate".into(),
        }
    }

    fn tests(&self, is_crate: bool, document: &str) -> Result<String> {
        Ok(self.handlebars.render(
            "integration_test",
            &json!({
                "is_crate": is_crate,
                "crate_name": self.crate_name(),
                "document": document,
            }),
        )?)
    }

    /// The generated module: header, enums, structs, collections.
    fn code(&mut self) -> Result<String> {
        self.assign_names();
        let mut out = String::new();
        out.push_str(&format!(
            "//! Generated by colap from `{}` – DO NOT EDIT\n\n#![allow(clippy::all)]\n\nuse std::collections::HashMap;\n\n/// The schema this code was generated from, as `Root::load` validates against it.\npub const SCHEMA: &str = {};\n\n",
            self.schema_path.display(),
            raw_string(&self.schema_text)
        ));
        let mut emitted = Emitted::default();
        self.collect(self.schema.root, "Root", true, &mut emitted);
        for e in &emitted.enums {
            out.push_str(&self.handlebars.render("enum", e)?);
            out.push('\n');
        }
        for s in &emitted.structs {
            out.push_str(&self.handlebars.render("struct", s)?);
            out.push('\n');
        }
        for (wrapper, element) in &emitted.collections {
            out.push_str(&self.handlebars.render(
                "collection",
                &json!({ "struct_name": wrapper, "singular_struct_name": element }),
            )?);
            out.push('\n');
        }
        Ok(out)
    }

    /// Name every shape, wrapper and enum before anything is emitted,
    /// so that a `like` declaration met before its target still shares
    /// its struct.
    fn assign_names(&mut self) {
        let mut names = Names::default();
        names.taken.insert("Root".into());
        names.struct_of.insert(self.schema.root, "Root".into());
        let mut stack: Vec<(ShapeId, String)> = vec![(self.schema.root, "Root".into())];
        let mut seen: HashSet<ShapeId> = HashSet::new();
        while let Some((shape, owner)) = stack.pop() {
            if !seen.insert(shape) {
                continue;
            }
            for decl in &self.schema.shape(shape).entities {
                let child_struct = match names.struct_of.get(&decl.shape) {
                    Some(name) => name.clone(),
                    None => {
                        let base = decl.name.to_pascal_case();
                        let name = names.claim([base.clone(), format!("{owner}{base}")]);
                        names.struct_of.insert(decl.shape, name.clone());
                        name
                    }
                };
                if let Some(plural) = &decl.plural {
                    let key = (decl.shape, plural.clone());
                    if !names.wrapper_of.contains_key(&key) {
                        let base = plural.to_pascal_case();
                        let name = names.claim([base.clone(), format!("{owner}{base}")]);
                        names.wrapper_of.insert(key, name);
                    }
                }
                stack.push((decl.shape, child_struct));
            }
        }
        self.names = names;
    }

    /// The enum a `symbol` field with `one_of` gets, claimed on first use.
    fn enum_for(&mut self, owner: &str, field: &FieldDecl) -> Option<(String, Vec<Value>)> {
        if field.ty != Type::Symbol {
            return None;
        }
        let symbols: Vec<&str> = field
            .one_of
            .as_ref()?
            .iter()
            .filter_map(|v| match v {
                ConfigValue::Symbol(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        if symbols.is_empty() {
            return None;
        }
        let name = self
            .names
            .claim([format!("{owner}{}", field.name.to_pascal_case())]);
        let mut variants = Vec::new();
        let mut used = HashSet::new();
        for symbol in symbols {
            let mut variant = symbol.to_pascal_case();
            if variant.is_empty() || variant.starts_with(|c: char| c.is_ascii_digit()) {
                variant = format!("V{variant}");
            }
            while !used.insert(variant.clone()) {
                variant.push('_');
            }
            variants.push(json!({ "variant": variant, "text": symbol }));
        }
        Some((name, variants))
    }

    fn collect(&mut self, shape: ShapeId, struct_name: &str, is_root: bool, out: &mut Emitted) {
        if !out.done.insert(shape) {
            return;
        }
        let mut fields = Vec::new();
        // Field declarations are read through the schema, which this
        // method borrows; the enum names are claimed on `self.names`.
        let field_count = self.schema.shape(shape).fields.len();
        for i in 0..field_count {
            let (name, original, doc, ty, list, required, one_of_symbols) = {
                let f = &self.schema.shape(shape).fields[i];
                (
                    field_ident(&f.name),
                    f.name.clone(),
                    f.doc.clone(),
                    f.ty,
                    f.list,
                    f.required,
                    f.one_of.clone(),
                )
            };
            let decl = FieldDecl {
                name: original.clone(),
                ty,
                list,
                required,
                one_of: one_of_symbols,
                min: None,
                max: None,
                pattern: None,
                to: None,
                doc: None,
                location: None,
            };
            let enum_name = match self.enum_for(struct_name, &decl) {
                Some((enum_name, variants)) => {
                    out.enums.push(json!({
                        "name": enum_name,
                        "doc": format!("`{original}` of `{struct_name}`: one of the schema's `one_of`."),
                        "variants": variants,
                    }));
                    Some(enum_name)
                }
                None => None,
            };
            let (element, read) = scalar_type(&decl, enum_name.as_deref());
            // `any` is always optional: `ConfigValue` has no default.
            let required = required && (list || ty != Type::Any);
            let inner = if list {
                format!("Vec<{element}>")
            } else {
                element.clone()
            };
            let rust_type = if required {
                inner.clone()
            } else {
                format!("Option<{inner}>")
            };
            fields.push(json!({
                "name": name,
                "original": original,
                "doc": doc,
                "rust_type": rust_type,
                "inner": inner,
                "list": list,
                "required": required,
                "read": read,
            }));
        }

        let mut children = Vec::new();
        let decls: Vec<Child> = self
            .schema
            .shape(shape)
            .entities
            .iter()
            .map(|d: &EntityDecl| Child {
                name: d.name.clone(),
                plural: d.plural.clone(),
                required: d.required,
                doc: d.doc.clone(),
                shape: d.shape,
            })
            .collect();
        for Child {
            name,
            plural,
            required,
            doc,
            shape: child_shape,
        } in decls
        {
            let child_struct = self
                .names
                .struct_of
                .get(&child_shape)
                .cloned()
                .unwrap_or_else(|| name.to_pascal_case());
            match &plural {
                Some(plural) => {
                    let wrapper = self
                        .names
                        .wrapper_of
                        .get(&(child_shape, plural.clone()))
                        .cloned()
                        .unwrap_or_else(|| plural.to_pascal_case());
                    if !out.collections.iter().any(|(w, _)| *w == wrapper) {
                        out.collections
                            .push((wrapper.clone(), child_struct.clone()));
                    }
                    children.push(json!({
                        "name": field_ident(plural),
                        "original": name,
                        "doc": doc,
                        "rust_type": wrapper,
                        "wrapper": wrapper,
                        "struct": child_struct,
                        "plural": true,
                        "required": true,
                    }));
                }
                None => {
                    let rust_type = if required {
                        child_struct.clone()
                    } else {
                        format!("Option<{child_struct}>")
                    };
                    children.push(json!({
                        "name": field_ident(&name),
                        "original": name,
                        "doc": doc,
                        "rust_type": rust_type,
                        "struct": child_struct,
                        "plural": false,
                        "required": required,
                    }));
                }
            }
            self.collect(child_shape, &child_struct, false, out);
        }

        out.structs.push(json!({
            "struct_name": struct_name,
            "is_root": is_root,
            "fields": fields,
            "children": children,
        }));
    }
}

/// A TOML basic string for a path.
fn raw_toml_string(path: &Path) -> String {
    format!(
        "\"{}\"",
        path.to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    )
}
