// SPDX-License-Identifier: Apache-2.0
//! The schema dialect as data: what a schema document declares, built
//! from its model once the meta-schema has accepted it, with the checks
//! no meta-schema can state (`docs/design/2026-10-31-schema-dialect.md`).

use crate::diagnostic::{Diagnostic, Severity};
use crate::model::config_model::{ConfigModel, ConfigValue, EntityNode, NodeId, Number};
use crate::model::source_location::SourceLocation;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A field's declared type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    String,
    Integer,
    Float,
    Number,
    Decimal,
    Boolean,
    Symbol,
    Reference,
    Instant,
    Date,
    Datetime,
    Edtf,
    Any,
}

impl Type {
    pub fn parse(symbol: &str) -> Option<Type> {
        Some(match symbol {
            "string" => Type::String,
            "integer" => Type::Integer,
            "float" => Type::Float,
            "number" => Type::Number,
            "decimal" => Type::Decimal,
            "boolean" => Type::Boolean,
            "symbol" => Type::Symbol,
            "reference" => Type::Reference,
            "instant" => Type::Instant,
            "date" => Type::Date,
            "datetime" => Type::Datetime,
            "edtf" => Type::Edtf,
            "any" => Type::Any,
            _ => return None,
        })
    }

    /// The type with its article, for messages: "an integer".
    pub fn described(self) -> &'static str {
        match self {
            Type::String => "a string",
            Type::Integer => "an integer",
            Type::Float => "a float",
            Type::Number => "a number",
            Type::Decimal => "a decimal",
            Type::Boolean => "a boolean",
            Type::Symbol => "a symbol",
            Type::Reference => "a reference",
            Type::Instant => "an instant",
            Type::Date => "a date",
            Type::Datetime => "a datetime",
            Type::Edtf => "an EDTF date",
            Type::Any => "anything",
        }
    }

    /// The literal kind a `one_of` entry of this type must have.
    pub fn admits_literal(self, value: &ConfigValue) -> bool {
        match (self, value) {
            (Type::Any, _) => true,
            (Type::String, ConfigValue::String(_)) => true,
            (Type::Symbol, ConfigValue::Symbol(_)) => true,
            (Type::Boolean, ConfigValue::Boolean(_)) => true,
            (Type::Reference, ConfigValue::Reference(_)) => true,
            (Type::Integer, ConfigValue::Number(n)) => n.is_integer(),
            (Type::Float | Type::Number | Type::Decimal, ConfigValue::Number(_)) => true,
            (Type::Instant | Type::Date | Type::Datetime | Type::Edtf, ConfigValue::String(_)) => {
                true
            }
            _ => false,
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            Type::Integer | Type::Float | Type::Number | Type::Decimal
        )
    }

    pub fn is_textual(self) -> bool {
        matches!(self, Type::String | Type::Symbol)
    }
}

/// What to do with a member the declaration does not name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unknown {
    #[default]
    Error,
    Warn,
    Allow,
}

impl Unknown {
    fn parse(symbol: &str) -> Option<Unknown> {
        Some(match symbol {
            "error" => Unknown::Error,
            "warn" => Unknown::Warn,
            "allow" => Unknown::Allow,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
    pub list: bool,
    pub required: bool,
    pub one_of: Option<Vec<ConfigValue>>,
    pub min: Option<Number>,
    pub max: Option<Number>,
    pub pattern: Option<Regex>,
    /// The path whose direct children a reference may point to.
    pub to: Option<String>,
    pub doc: Option<String>,
    pub location: Option<SourceLocation>,
}

/// Index of a [`Shape`] in [`Schema::shapes`].
pub type ShapeId = usize;

#[derive(Debug)]
pub struct EntityDecl {
    pub name: String,
    pub plural: Option<String>,
    pub required: bool,
    pub unknown: Unknown,
    pub key_pattern: Option<Regex>,
    pub doc: Option<String>,
    /// Its own shape, or the one `like` names.
    pub shape: ShapeId,
    pub location: Option<SourceLocation>,
}

impl EntityDecl {
    /// `name` or `name plural plural`, as the document must write it.
    pub fn declaration(&self) -> String {
        match &self.plural {
            Some(plural) => format!("{} plural {plural}", self.name),
            None => self.name.clone(),
        }
    }
}

/// The fields and child entities a declaration gives an entity.
#[derive(Debug, Default)]
pub struct Shape {
    pub fields: Vec<FieldDecl>,
    pub entities: Vec<EntityDecl>,
}

#[derive(Debug)]
pub struct Schema {
    pub name: Option<String>,
    pub version: Option<i64>,
    /// Whether the `_utc` naming rule is checked.
    pub utc_suffix: bool,
    /// What to do with a root entity the schema does not name.
    pub unknown: Unknown,
    pub shapes: Vec<Shape>,
    /// The shape of the root: its entities are the top-level
    /// declarations.
    pub root: ShapeId,
}

impl Schema {
    pub fn shape(&self, id: ShapeId) -> &Shape {
        self.shapes.get(id).unwrap_or(&EMPTY)
    }
}

static EMPTY: Shape = Shape {
    fields: Vec::new(),
    entities: Vec::new(),
};

/// What `build` found while reading declarations, by severity.
struct Findings {
    path: Option<PathBuf>,
    out: Vec<Diagnostic>,
}

impl Findings {
    fn error(&mut self, location: Option<&SourceLocation>, message: String) {
        self.out.push(Diagnostic::at(
            Severity::Error,
            self.path.as_deref(),
            location,
            message,
        ));
    }

    fn warning(&mut self, location: Option<&SourceLocation>, message: String) {
        self.out.push(Diagnostic::at(
            Severity::Warning,
            self.path.as_deref(),
            location,
            message,
        ));
    }
}

/// A pending `like`, resolved once every declaration has a shape.
struct Like {
    shape: ShapeId,
    target: String,
    location: Option<SourceLocation>,
    owner: String,
}

struct Builder<'a> {
    model: &'a ConfigModel,
    shapes: Vec<Shape>,
    /// Declaration entry node to its shape.
    by_node: HashMap<NodeId, ShapeId>,
    likes: Vec<Like>,
    utc_suffix: bool,
    findings: Findings,
}

fn string_field(node: &EntityNode, name: &str) -> Option<String> {
    match node.field_value(name)? {
        ConfigValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn symbol_field(node: &EntityNode, name: &str) -> Option<String> {
    match node.field_value(name)? {
        ConfigValue::Symbol(s) => Some(s.clone()),
        _ => None,
    }
}

fn bool_field(node: &EntityNode, name: &str) -> Option<bool> {
    match node.field_value(name)? {
        ConfigValue::Boolean(b) => Some(*b),
        _ => None,
    }
}

fn number_field(node: &EntityNode, name: &str) -> Option<Number> {
    match node.field_value(name)? {
        ConfigValue::Number(n) => Some(n.clone()),
        _ => None,
    }
}

fn field_location(node: &EntityNode, name: &str) -> Option<SourceLocation> {
    node.field(name).and_then(|f| f.location.clone())
}

/// Build the schema a model declares. The model has already matched
/// the meta-schema, so every field has the right kind; what is
/// checked here is what the meta-schema cannot say. The schema comes
/// back unless an error was found; warnings come back either way.
pub(crate) fn build(model: &ConfigModel, path: Option<&Path>) -> (Option<Schema>, Vec<Diagnostic>) {
    let mut builder = Builder {
        model,
        shapes: Vec::new(),
        by_node: HashMap::new(),
        likes: Vec::new(),
        utc_suffix: true,
        findings: Findings {
            path: path.map(Path::to_path_buf),
            out: Vec::new(),
        },
    };
    let (name, version, unknown) = {
        let Some(schema_id) = model.find_entity_by_path("schema") else {
            builder
                .findings
                .error(None, "a schema document declares a `schema` entity".into());
            return (None, builder.findings.out);
        };
        let Some(node) = model.get_node(schema_id) else {
            return (None, builder.findings.out);
        };
        let node = node.borrow();
        builder.utc_suffix = bool_field(&node, "utc_suffix").unwrap_or(true);
        (
            string_field(&node, "name"),
            number_field(&node, "version").and_then(|n| n.as_i64()),
            symbol_field(&node, "unknown")
                .and_then(|s| Unknown::parse(&s))
                .unwrap_or_default(),
        )
    };
    let root = builder.shapes.len();
    builder.shapes.push(Shape::default());
    let entities = model
        .find_entity_by_path("schema")
        .map(|id| builder.declarations(id, ""))
        .unwrap_or_default();
    builder.shapes[root].entities = entities;
    builder.resolve_likes();

    let Builder {
        shapes,
        utc_suffix,
        findings,
        ..
    } = builder;
    let errors = findings.out.iter().any(|d| d.severity == Severity::Error);
    let schema = (!errors).then_some(Schema {
        name,
        version,
        utc_suffix,
        unknown,
        shapes,
        root,
    });
    (schema, findings.out)
}

impl Builder<'_> {
    /// The entity declarations under `owner`'s `entity plural entities`.
    fn declarations(&mut self, owner: NodeId, owner_path: &str) -> Vec<EntityDecl> {
        let Some(owner_node) = self.model.get_node(owner) else {
            return Vec::new();
        };
        let Some(collection) = owner_node.borrow().child("entity") else {
            return Vec::new();
        };
        let Some(collection) = self.model.get_node(collection) else {
            return Vec::new();
        };
        let entries: Vec<NodeId> = collection.borrow().children.clone();
        entries
            .into_iter()
            .filter_map(|entry| self.declaration(entry, owner_path))
            .collect()
    }

    fn declaration(&mut self, entry: NodeId, owner_path: &str) -> Option<EntityDecl> {
        let node = self.model.get_node(entry)?;
        let node = node.borrow();
        let name = node.name.clone();
        let decl_path = if owner_path.is_empty() {
            name.clone()
        } else {
            format!("{owner_path}/{name}")
        };
        let plural = symbol_field(&node, "plural");
        let key_pattern = string_field(&node, "key_pattern").and_then(|p| {
            if plural.is_none() {
                self.findings.error(
                    field_location(&node, "key_pattern").as_ref(),
                    format!("key_pattern of `{decl_path}` applies to collections"),
                );
                return None;
            }
            self.regex(
                &p,
                &format!("key_pattern of `{decl_path}`"),
                field_location(&node, "key_pattern"),
            )
        });

        let shape = self.shapes.len();
        self.shapes.push(Shape::default());
        self.by_node.insert(entry, shape);

        let like = node.field_value("like").and_then(|v| match v {
            ConfigValue::Reference(target) => Some(target.clone()),
            _ => None,
        });
        let has_own = node.child("field").is_some() || node.child("entity").is_some();
        match like {
            Some(target) => {
                if has_own {
                    self.findings.error(
                        field_location(&node, "like").as_ref(),
                        format!(
                            "`{decl_path}` has `like` and declares fields or entities of its own"
                        ),
                    );
                }
                self.likes.push(Like {
                    shape,
                    target,
                    location: field_location(&node, "like"),
                    owner: decl_path.clone(),
                });
            }
            None => {
                let fields = self.fields(&node, &decl_path);
                drop(node);
                let entities = self.declarations(entry, &decl_path);
                self.shapes[shape].fields = fields;
                self.shapes[shape].entities = entities;
                let node = self.model.get_node(entry)?;
                let node = node.borrow();
                return Some(EntityDecl {
                    name,
                    plural,
                    required: bool_field(&node, "required").unwrap_or(false),
                    unknown: symbol_field(&node, "unknown")
                        .and_then(|s| Unknown::parse(&s))
                        .unwrap_or_default(),
                    key_pattern,
                    doc: string_field(&node, "doc"),
                    shape,
                    location: node.location.clone(),
                });
            }
        }
        Some(EntityDecl {
            name,
            plural,
            required: bool_field(&node, "required").unwrap_or(false),
            unknown: symbol_field(&node, "unknown")
                .and_then(|s| Unknown::parse(&s))
                .unwrap_or_default(),
            key_pattern,
            doc: string_field(&node, "doc"),
            shape,
            location: node.location.clone(),
        })
    }

    /// The field declarations under a declaration's `field plural fields`.
    fn fields(&mut self, decl: &EntityNode, decl_path: &str) -> Vec<FieldDecl> {
        let Some(collection) = decl.child("field") else {
            return Vec::new();
        };
        let Some(collection) = self.model.get_node(collection) else {
            return Vec::new();
        };
        let entries: Vec<NodeId> = collection.borrow().children.clone();
        entries
            .into_iter()
            .filter_map(|entry| self.field(entry, decl_path))
            .collect()
    }

    fn field(&mut self, entry: NodeId, decl_path: &str) -> Option<FieldDecl> {
        let node = self.model.get_node(entry)?;
        let node = node.borrow();
        let name = node.name.clone();
        let ty = symbol_field(&node, "type")
            .and_then(|s| Type::parse(&s))
            .unwrap_or(Type::Any);
        let at = |field: &str| field_location(&node, field);
        let about = |field: &str| format!("{field} of `{name}` in `{decl_path}`");

        if self.utc_suffix {
            if name.ends_with("_utc") && ty != Type::Instant {
                self.findings.error(
                    node.location.as_ref(),
                    format!(
                        "field `{name}` of `{decl_path}` ends in `_utc` and must be an instant, not {}",
                        ty.described()
                    ),
                );
            } else if ty == Type::Instant && !name.ends_with("_utc") {
                self.findings.warning(
                    node.location.as_ref(),
                    format!(
                        "field `{name}` of `{decl_path}` is an instant and should end in `_utc`"
                    ),
                );
            }
        }

        let one_of = node.field_value("one_of").map(|v| match v {
            ConfigValue::List(items) => items.clone(),
            scalar => vec![scalar.clone()],
        });
        if let Some(items) = &one_of {
            for item in items {
                if !ty.admits_literal(item) {
                    self.findings.error(
                        at("one_of").as_ref(),
                        format!(
                            "{} holds {item}, which is not {}",
                            about("one_of"),
                            ty.described()
                        ),
                    );
                }
            }
        }

        let min = number_field(&node, "min");
        let max = number_field(&node, "max");
        for (bound, value) in [("min", &min), ("max", &max)] {
            if value.is_some() && !ty.is_numeric() {
                self.findings.error(
                    at(bound).as_ref(),
                    format!(
                        "{} applies to numbers, not {}",
                        about(bound),
                        ty.described()
                    ),
                );
            }
        }
        if let (Some(lo), Some(hi)) = (&min, &max)
            && lo > hi
        {
            self.findings.error(
                at("min").as_ref(),
                format!("{} is above its max", about("min")),
            );
        }

        let pattern = string_field(&node, "pattern").and_then(|p| {
            if !ty.is_textual() {
                self.findings.error(
                    at("pattern").as_ref(),
                    format!(
                        "{} applies to strings and symbols, not {}",
                        about("pattern"),
                        ty.described()
                    ),
                );
                return None;
            }
            self.regex(&p, &about("pattern"), at("pattern"))
        });

        let to = string_field(&node, "to");
        if to.is_some() && ty != Type::Reference {
            self.findings.error(
                at("to").as_ref(),
                format!(
                    "{} applies to references, not {}",
                    about("to"),
                    ty.described()
                ),
            );
        }

        Some(FieldDecl {
            name,
            ty,
            list: bool_field(&node, "list").unwrap_or(false),
            required: bool_field(&node, "required").unwrap_or(false),
            one_of,
            min,
            max,
            pattern,
            to,
            doc: string_field(&node, "doc"),
            location: node.location.clone(),
        })
    }

    /// A pattern anchored to the whole value.
    fn regex(
        &mut self,
        pattern: &str,
        what: &str,
        location: Option<SourceLocation>,
    ) -> Option<Regex> {
        match Regex::new(&format!("^(?:{pattern})$")) {
            Ok(re) => Some(re),
            Err(e) => {
                // The regex crate's message spans lines; its last line
                // names the problem.
                let reason = e
                    .to_string()
                    .lines()
                    .last()
                    .unwrap_or_default()
                    .trim_start_matches("error: ")
                    .to_string();
                self.findings.error(
                    location.as_ref(),
                    format!("{what} is not a valid regular expression: {reason}"),
                );
                None
            }
        }
    }

    /// Point every `like` at its target's shape, now that all exist.
    fn resolve_likes(&mut self) {
        let likes = std::mem::take(&mut self.likes);
        for like in likes {
            let target = self
                .model
                .find_entity_by_path(&like.target)
                .and_then(|id| self.by_node.get(&id).copied());
            match target {
                Some(target_shape) if target_shape != like.shape => {
                    // The shape is shared: the `like` declaration's own
                    // (empty) shape is replaced by an alias.
                    self.alias(like.shape, target_shape);
                }
                _ => self.findings.error(
                    like.location.as_ref(),
                    format!(
                        "like of `{}` must point to another declaration, not `@{}`",
                        like.owner, like.target
                    ),
                ),
            }
        }
    }

    /// Every declaration whose shape is `from` now uses `to`.
    fn alias(&mut self, from: ShapeId, to: ShapeId) {
        for shape in &mut self.shapes {
            for decl in &mut shape.entities {
                if decl.shape == from {
                    decl.shape = to;
                }
            }
        }
    }
}
