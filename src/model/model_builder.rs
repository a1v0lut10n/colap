// SPDX-License-Identifier: Apache-2.0
//! From the grammar's AST to a [`ConfigModel`].
//!
//! A literate file holds several cola blocks, each parsed on its own
//! because the grammar only ever sees a fence body. The builder is
//! therefore incremental: [`ModelBuilder::new`], one
//! [`ModelBuilder::add_block`] per block with the block's line offset
//! in the file, then [`ModelBuilder::finish`]. Every
//! [`SourceLocation`] it records points into the original file.
//!
//! The builder does not stop at the first problem. Every duplicate the
//! model rejects is recorded (a rejected entity's subtree is skipped)
//! and `finish` returns them all, so one file read reports every
//! error in it. `finish` also resolves every `@path` reference against
//! the finished model, so a reference may point forward, or into
//! another block.

use crate::error::Error;
use crate::model::config_model::{ConfigModel, ConfigValue, ModelError, NodeId};
use crate::model::literal;
use crate::model::source_location::SourceLocation;
use crate::parser::cola_actions::{Cola, Entity, Field, FieldValue, Item, List, Scalar};
use std::path::{Path, PathBuf};

/// Builds a ConfigModel from parsed cola blocks.
pub struct ModelBuilder {
    model: ConfigModel,
    path: Option<PathBuf>,
    line_offset: u32,
    errors: Vec<Error>,
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ModelBuilder {
    /// Start an empty model; `path` is recorded on every location.
    pub fn new(path: Option<&Path>) -> Self {
        Self {
            model: ConfigModel::new(),
            path: path.map(Path::to_path_buf),
            line_offset: 0,
            errors: Vec::new(),
        }
    }

    /// Add the entities of one parsed block. `line_offset` is the
    /// number of file lines above the block body's first line, so a
    /// body line `n` is file line `n + line_offset`. Problems are
    /// recorded, not returned; see [`ModelBuilder::finish`].
    pub fn add_block(&mut self, cola: &Cola, line_offset: u32) {
        self.line_offset = line_offset;
        let root_id = self.model.root_id();
        if let Some(entities) = cola {
            for entity in entities {
                self.process_entity(root_id, entity);
            }
        }
    }

    /// Record an error found outside the builder, such as a block that
    /// did not parse, so that `finish` reports it with the others.
    pub fn record(&mut self, error: Error) {
        self.errors.push(error);
    }

    /// The errors recorded so far.
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// The finished model, or every error recorded while building it,
    /// including every reference that resolves to no entity.
    pub fn finish(mut self) -> Result<ConfigModel, Error> {
        self.resolve_references();
        Error::collect(self.errors)?;
        Ok(self.model)
    }

    /// The model of the blocks added so far, with references left
    /// unresolved: for one block of a document whose other blocks the
    /// references may point into, when the whole document has already
    /// been checked.
    pub fn finish_block(self) -> Result<ConfigModel, Error> {
        Error::collect(self.errors)?;
        Ok(self.model)
    }

    fn resolve_references(&mut self) {
        let mut unresolved = Vec::new();
        for id in self.model.node_ids() {
            let Some(node) = self.model.get_node(id) else {
                continue;
            };
            for field in &node.borrow().fields {
                for value in field.value.scalars() {
                    if let ConfigValue::Reference(path) = value
                        && self.model.find_entity_by_path(path).is_none()
                    {
                        unresolved.push(ModelError::UnresolvedReference {
                            path: path.clone(),
                            location: field.location.clone(),
                        });
                    }
                }
            }
        }
        for error in unresolved {
            self.reject(error);
        }
    }

    /// Convert a single parsed block to a ConfigModel, with locations
    /// relative to that block. The incremental API above is the one to
    /// use for literate files; see also [`crate::parse_str`].
    pub fn build_config_model(cola: &Cola) -> Result<ConfigModel, Error> {
        let mut builder = Self::new(None);
        builder.add_block(cola, 0);
        builder.finish()
    }

    /// A rustemo location lifted to the file's coordinates.
    fn location(&self, loc: Option<&rustemo::Location>) -> Option<SourceLocation> {
        let loc = loc?;
        let (start_line, start_column) = (loc.start.line() as u32, loc.start.column() as u32);
        let (end_line, end_column) = loc
            .end
            .map(|end| (end.line() as u32, end.column() as u32))
            .unwrap_or((start_line, start_column));
        Some(SourceLocation {
            file_path: self.path.clone().unwrap_or_default(),
            start_line: start_line + self.line_offset,
            start_column,
            end_line: end_line + self.line_offset,
            end_column,
        })
    }

    fn reject(&mut self, error: ModelError) {
        self.errors.push(Error::model(error, self.path.as_deref()));
    }

    /// Process an entity and add it to the model.
    fn process_entity(&mut self, parent_id: NodeId, entity: &Entity) {
        let (entity_name, plural_name, location, definition) = match entity {
            Entity::SingularEntity(singular) => (
                singular.identifier.as_ref().trim(),
                None,
                self.location(singular.location.as_ref()),
                &singular.entity_definition,
            ),
            Entity::PluralEntity(plural) => (
                plural.identifier_1.as_ref().trim(),
                Some(plural.identifier_3.as_ref().trim()),
                self.location(plural.location.as_ref()),
                &plural.entity_definition,
            ),
        };

        let entity_id = match self
            .model
            .add_entity(parent_id, entity_name, plural_name, location)
        {
            Ok(id) => id,
            Err(error) => {
                // A rejected entity takes its subtree with it: there is
                // no node to attach the contents to.
                self.reject(error);
                return;
            }
        };

        for item in definition.iter().flatten() {
            match item {
                Item::Field1(field) | Item::Field2(field) => {
                    self.add_field_to_entity(entity_id, field);
                }
                Item::Entity(entity) => self.process_entity(entity_id, entity),
            }
        }
    }

    /// Add a field to an entity in the model.
    fn add_field_to_entity(&mut self, entity_id: NodeId, field: &Field) {
        let field_name = field.identifier.as_ref().trim().to_string();
        let location = self.location(field.location.as_ref());
        let field_value = match Self::convert_field_value(&field.field_value) {
            Ok(value) => value,
            Err(message) => {
                self.reject(ModelError::InvalidValue { message, location });
                return;
            }
        };
        if let Err(error) = self
            .model
            .add_field(entity_id, &field_name, field_value, location)
        {
            self.reject(error);
        }
    }

    /// Convert a FieldValue from the AST to a ConfigValue for the model.
    fn convert_field_value(field_value: &FieldValue) -> Result<ConfigValue, String> {
        match field_value {
            FieldValue::Scalar(scalar) => Self::convert_scalar(scalar),
            FieldValue::List(List::Scalar0(items)) => items
                .iter()
                .flatten()
                .map(Self::convert_scalar)
                .collect::<Result<Vec<_>, _>>()
                .map(ConfigValue::List),
            FieldValue::List(List::Scalar1(items)) => items
                .iter()
                .map(Self::convert_scalar)
                .collect::<Result<Vec<_>, _>>()
                .map(ConfigValue::List),
        }
    }

    fn convert_scalar(scalar: &Scalar) -> Result<ConfigValue, String> {
        match scalar {
            Scalar::QuotedStringDouble(s) | Scalar::QuotedStringSingle(s) => {
                let raw = s.as_ref().trim();
                literal::decode_quoted(&raw[1..raw.len() - 1]).map(ConfigValue::String)
            }
            Scalar::BlockString(s) => {
                let raw = s.as_ref().trim();
                Ok(ConfigValue::String(literal::decode_block(
                    &raw[3..raw.len() - 3],
                )))
            }
            Scalar::Number(n) => literal::parse_number(n.as_ref().trim()),
            Scalar::BooleanTrue => Ok(ConfigValue::Boolean(true)),
            Scalar::BooleanFalse => Ok(ConfigValue::Boolean(false)),
            Scalar::Reference(r) => Ok(ConfigValue::Reference(r.as_ref().trim()[1..].to_string())),
            Scalar::Identifier(i) => Ok(ConfigValue::Symbol(i.as_ref().trim().to_string())),
        }
    }
}
