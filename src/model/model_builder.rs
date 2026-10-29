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
//! error in it.

use crate::error::Error;
use crate::model::config_model::{ConfigModel, ConfigValue, ModelError, NodeId};
use crate::model::source_location::SourceLocation;
use crate::parser::cola_actions::{Cola, Entity, Field, FieldList, FieldValue, NestedBlock};
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

    /// The finished model, or every error recorded while building it.
    pub fn finish(self) -> Result<ConfigModel, Error> {
        Error::collect(self.errors)?;
        Ok(self.model)
    }

    /// Convert a single parsed block to a ConfigModel, with locations
    /// relative to that block. The incremental API above is the one to
    /// use for literate files; see also [`crate::parse_str`].
    pub fn build_config_model(cola: &Cola) -> Result<ConfigModel, String> {
        let mut builder = Self::new(None);
        builder.add_block(cola, 0);
        builder.finish().map_err(|e| e.to_string())
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

        if let Some(nested_blocks) = definition {
            for nested_block in nested_blocks {
                match nested_block {
                    NestedBlock::FieldList(field_list) => {
                        self.process_field_list(entity_id, field_list);
                    }
                    NestedBlock::Entity(entity) => {
                        self.process_entity(entity_id, entity);
                    }
                }
            }
        }
    }

    /// Process a field list and add fields to the entity.
    fn process_field_list(&mut self, entity_id: NodeId, field_list: &FieldList) {
        match field_list {
            FieldList::Field(field) => self.add_field_to_entity(entity_id, field),
            FieldList::C2(c2) => {
                self.process_field_list(entity_id, &c2.field_list);
                self.add_field_to_entity(entity_id, &c2.field);
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
            FieldValue::QuotedStringDouble(s) | FieldValue::QuotedStringSingle(s) => {
                // Strip the surrounding quotes; escapes stay raw for now
                // (see the value-language task).
                let s_val = s.as_ref().trim();
                Ok(ConfigValue::String(s_val[1..s_val.len() - 1].to_string()))
            }
            FieldValue::Number(n) => {
                let n_str = n.as_ref().trim();
                if n_str.contains('.') {
                    n_str
                        .parse::<f64>()
                        .map(ConfigValue::Float)
                        .map_err(|_| format!("Failed to parse float: {n_str}"))
                } else {
                    n_str
                        .parse::<i64>()
                        .map(ConfigValue::Integer)
                        .map_err(|_| format!("Failed to parse integer: {n_str}"))
                }
            }
            FieldValue::BooleanTrue => Ok(ConfigValue::Boolean(true)),
            FieldValue::BooleanFalse => Ok(ConfigValue::Boolean(false)),
        }
    }
}
