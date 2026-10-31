// SPDX-License-Identifier: Apache-2.0
//! Validating a document's model against a [`Schema`]: the walk and
//! the findings of `docs/design/2026-10-31-schema-dialect.md`.

use super::dialect::{EntityDecl, FieldDecl, Schema, ShapeId, Type, Unknown};
use super::temporal;
use crate::diagnostic::{Diagnostic, Severity};
use crate::model::canonical::value_text;
use crate::model::config_model::{ConfigField, ConfigModel, ConfigValue, NodeId};
use crate::model::source_location::SourceLocation;
use std::path::{Path, PathBuf};

struct Validator<'a> {
    schema: &'a Schema,
    model: &'a ConfigModel,
    path: Option<PathBuf>,
    out: Vec<Diagnostic>,
}

/// Every finding of `model` against `schema`, in line order.
pub(crate) fn validate(
    schema: &Schema,
    model: &ConfigModel,
    path: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut v = Validator {
        schema,
        model,
        path: path.map(Path::to_path_buf),
        out: Vec::new(),
    };
    v.shape(schema.root, model.root_id(), schema.unknown);
    v.out
        .sort_by_key(|d| (d.line.unwrap_or(0), d.column.unwrap_or(0)));
    v.out
}

/// How an entity is named in messages.
fn place(model: &ConfigModel, id: NodeId) -> String {
    let path = model.entity_path(id);
    if path.is_empty() {
        "the root".to_string()
    } else {
        format!("`{path}`")
    }
}

impl Validator<'_> {
    fn finding(&mut self, severity: Severity, location: Option<&SourceLocation>, message: String) {
        self.out.push(Diagnostic::at(
            severity,
            self.path.as_deref(),
            location,
            message,
        ));
    }

    fn error(&mut self, location: Option<&SourceLocation>, message: String) {
        self.finding(Severity::Error, location, message);
    }

    fn unknown(&mut self, policy: Unknown, location: Option<&SourceLocation>, message: String) {
        match policy {
            Unknown::Error => self.finding(Severity::Error, location, message),
            Unknown::Warn => self.finding(Severity::Warning, location, message),
            Unknown::Allow => {}
        }
    }

    /// Check the entity `id` against the shape: its fields, its child
    /// entities, and whatever it holds that the shape does not name.
    fn shape(&mut self, shape: ShapeId, id: NodeId, policy: Unknown) {
        let Some(node) = self.model.get_node(id) else {
            return;
        };
        let node = node.borrow();
        let here = place(self.model, id);
        let shape = self.schema.shape(shape);

        for decl in &shape.fields {
            match node.field(&decl.name) {
                Some(field) => self.field(decl, field, &here),
                None if decl.required => self.error(
                    node.location.as_ref(),
                    format!("missing required field `{}` in {here}", decl.name),
                ),
                None => {}
            }
        }
        for field in &node.fields {
            if !shape.fields.iter().any(|d| d.name == field.name) {
                self.unknown(
                    policy,
                    field.location.as_ref(),
                    format!("unknown field `{}` in {here}", field.name),
                );
            }
        }

        for decl in &shape.entities {
            match node.child(&decl.name) {
                Some(child) => self.entity(decl, child, &here),
                None if decl.required => self.error(
                    node.location.as_ref(),
                    format!("missing required entity `{}` in {here}", decl.name),
                ),
                None => {}
            }
        }
        for &child in &node.children {
            let Some(child_node) = self.model.get_node(child) else {
                continue;
            };
            let child_node = child_node.borrow();
            let declared = shape.entities.iter().any(|d| {
                d.name == child_node.name
                    || d.plural
                        .as_deref()
                        .is_some_and(|p| Some(p) == child_node.plural_name.as_deref())
            });
            if !declared {
                self.unknown(
                    policy,
                    child_node.location.as_ref(),
                    format!("unknown entity `{}` in {here}", child_node.name),
                );
            }
        }
    }

    /// A child entity against its declaration: the right kind, then
    /// the entries of a collection or the entity itself.
    fn entity(&mut self, decl: &EntityDecl, id: NodeId, parent: &str) {
        let Some(node) = self.model.get_node(id) else {
            return;
        };
        let node = node.borrow();
        if node.plural_name != decl.plural {
            self.error(
                node.location.as_ref(),
                format!(
                    "entity `{}` in {parent} is `{}`, but the schema declares `{}`",
                    node.name,
                    node.declaration(),
                    decl.declaration()
                ),
            );
            return;
        }
        if decl.plural.is_some() {
            let entries = node.children.clone();
            let collection = place(self.model, id);
            drop(node);
            for entry in entries {
                let Some(entry_node) = self.model.get_node(entry) else {
                    continue;
                };
                let (key, location) = {
                    let n = entry_node.borrow();
                    (n.name.clone(), n.location.clone())
                };
                if let Some(pattern) = &decl.key_pattern
                    && !pattern.is_match(&key)
                {
                    self.error(
                        location.as_ref(),
                        format!(
                            "key `{key}` in {collection} does not match pattern `{}`",
                            trim_anchors(pattern.as_str())
                        ),
                    );
                }
                self.shape(decl.shape, entry, decl.unknown);
            }
        } else {
            drop(node);
            self.shape(decl.shape, id, decl.unknown);
        }
    }

    /// A field's value against its declaration.
    fn field(&mut self, decl: &FieldDecl, field: &ConfigField, here: &str) {
        let location = field.location.as_ref();
        let name = &field.name;
        match (&field.value, decl.list) {
            // `any` without `list` takes anything, a list included.
            (_, false) if decl.ty == Type::Any => {}
            (ConfigValue::List(items), true) => {
                for item in items {
                    if let Some(problem) = self.scalar(decl, item, name, here) {
                        self.error(location, problem);
                    }
                }
            }
            (ConfigValue::List(_), false) => self.error(
                location,
                format!(
                    "field `{name}` in {here} must be {}, got {}",
                    decl.ty.described(),
                    value_text(&field.value)
                ),
            ),
            (scalar, true) => self.error(
                location,
                format!(
                    "field `{name}` in {here} must be a list of {}, got {}",
                    decl.ty.described().split_once(' ').map_or("", |(_, t)| t),
                    value_text(scalar)
                ),
            ),
            (scalar, false) => {
                if let Some(problem) = self.scalar(decl, scalar, name, here) {
                    self.error(location, problem);
                }
            }
        }
        // The one warning: an instant in a `_utc` field not written as `Z`.
        if self.schema.utc_suffix && decl.ty == Type::Instant && name.ends_with("_utc") {
            for value in field.value.scalars() {
                if let ConfigValue::String(s) = value
                    && temporal::instant(s) == Some(false)
                {
                    let message = format!(
                        "value {} of `{name}` in {here} is not written in UTC",
                        value_text(value)
                    );
                    self.finding(Severity::Warning, location, message);
                }
            }
        }
    }

    /// The problem with one scalar against the declaration, if any.
    fn scalar(
        &self,
        decl: &FieldDecl,
        value: &ConfigValue,
        name: &str,
        here: &str,
    ) -> Option<String> {
        let shown = value_text(value);
        let wrong_kind = || {
            Some(format!(
                "field `{name}` in {here} must be {}, got {shown}",
                decl.ty.described()
            ))
        };
        let about = |what: &str| format!("value {shown} of `{name}` in {here} {what}");
        match (decl.ty, value) {
            (Type::Any, _) => {}
            (Type::String, ConfigValue::String(_)) => {}
            (Type::Symbol, ConfigValue::Symbol(_)) => {}
            (Type::Boolean, ConfigValue::Boolean(_)) => {}
            (Type::Integer, ConfigValue::Number(n)) => {
                if !n.is_integer() {
                    return wrong_kind();
                }
                if n.as_i64().is_none() {
                    return Some(format!(
                        "field `{name}` in {here} must fit a 64-bit integer, got {shown}"
                    ));
                }
            }
            (Type::Float | Type::Number | Type::Decimal, ConfigValue::Number(_)) => {}
            (Type::Reference, ConfigValue::Reference(target)) => {
                if let Some(to) = &decl.to {
                    let parent = self.model.find_entity_by_path(to);
                    let actual = self
                        .model
                        .find_entity_by_path(target)
                        .and_then(|id| self.model.get_node(id))
                        .and_then(|n| n.borrow().parent);
                    if parent.is_none() || actual != parent {
                        return Some(about(&format!("must point to an entry of `{to}`")));
                    }
                }
            }
            (Type::Instant, ConfigValue::String(s)) => {
                if temporal::instant(s).is_none() {
                    return Some(about("is not an instant"));
                }
            }
            (Type::Date, ConfigValue::String(s)) => {
                if !temporal::date(s) {
                    return Some(about("is not a date"));
                }
            }
            (Type::Datetime, ConfigValue::String(s)) => {
                if !temporal::datetime(s) {
                    return Some(about("is not a datetime"));
                }
            }
            (Type::Edtf, ConfigValue::String(s)) => {
                if !temporal::edtf(s) {
                    return Some(about("is not an EDTF date"));
                }
            }
            _ => return wrong_kind(),
        }

        if let Some(allowed) = &decl.one_of
            && !allowed.contains(value)
        {
            let list: Vec<String> = allowed.iter().map(value_text).collect();
            return Some(about(&format!("is not one of [{}]", list.join(", "))));
        }
        if let ConfigValue::Number(n) = value {
            if let Some(min) = &decl.min
                && n < min
            {
                return Some(about(&format!("is below the minimum {min}")));
            }
            if let Some(max) = &decl.max
                && n > max
            {
                return Some(about(&format!("is above the maximum {max}")));
            }
        }
        if let Some(pattern) = &decl.pattern {
            let text = match value {
                ConfigValue::String(s) | ConfigValue::Symbol(s) => Some(s.as_str()),
                _ => None,
            };
            if let Some(text) = text
                && !pattern.is_match(text)
            {
                return Some(about(&format!(
                    "does not match pattern `{}`",
                    trim_anchors(pattern.as_str())
                )));
            }
        }
        None
    }
}

/// The pattern as the schema wrote it, without the anchors the loader
/// added.
fn trim_anchors(anchored: &str) -> &str {
    anchored
        .strip_prefix("^(?:")
        .and_then(|s| s.strip_suffix(")$"))
        .unwrap_or(anchored)
}
