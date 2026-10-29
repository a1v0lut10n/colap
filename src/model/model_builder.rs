// SPDX-License-Identifier: Apache-2.0
//! From the grammar's AST to a [`ConfigModel`].
//!
//! A literate file holds several cola blocks, each parsed on its own
//! because the grammar only ever sees a fence body. The builder is
//! therefore incremental: [`ModelBuilder::new`], one
//! [`ModelBuilder::add_block`] per block with the block's line offset
//! in the file, then [`ModelBuilder::finish`]. Every
//! [`SourceLocation`] it records points into the original file.

use crate::error::Error;
use crate::model::config_model::{ConfigModel, ConfigValue};
use crate::model::source_location::SourceLocation;
use crate::parser::cola_actions::{Cola, Entity, Field, FieldList, FieldValue, NestedBlock};
use std::path::{Path, PathBuf};

/// Builds a ConfigModel from parsed cola blocks.
pub struct ModelBuilder {
    model: ConfigModel,
    path: Option<PathBuf>,
    line_offset: u32,
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
        }
    }

    /// Add the entities of one parsed block. `line_offset` is the
    /// number of file lines above the block body's first line, so a
    /// body line `n` is file line `n + line_offset`.
    pub fn add_block(&mut self, cola: &Cola, line_offset: u32) -> Result<(), Error> {
        self.line_offset = line_offset;
        let root_id = self.model.root_id();
        if let Some(entities) = cola {
            for entity in entities {
                self.process_entity(root_id, "", entity)
                    .map_err(|message| Error::model(message, self.path.as_deref()))?;
            }
        }
        Ok(())
    }

    /// The finished model.
    pub fn finish(self) -> ConfigModel {
        self.model
    }

    /// Convert a single parsed block to a ConfigModel, with locations
    /// relative to that block. The incremental API above is the one to
    /// use for literate files; see also [`crate::parse_str`].
    pub fn build_config_model(cola: &Cola) -> Result<ConfigModel, String> {
        let mut builder = Self::new(None);
        builder.add_block(cola, 0).map_err(|e| e.to_string())?;
        Ok(builder.finish())
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

    /// Process an entity and add it to the model.
    fn process_entity(
        &mut self,
        _parent_id: usize,
        parent_path: &str,
        entity: &Entity,
    ) -> Result<(), String> {
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

        let path = if parent_path.is_empty() {
            entity_name.to_string()
        } else {
            format!("{parent_path}/{entity_name}")
        };

        let entity_id =
            self.model
                .create_entity_at_path(parent_path, entity_name, plural_name, location)?;

        if let Some(nested_blocks) = definition {
            for nested_block in nested_blocks {
                match nested_block {
                    NestedBlock::FieldList(field_list) => {
                        self.process_field_list(entity_id, field_list)?;
                    }
                    NestedBlock::Entity(entity) => {
                        self.process_entity(entity_id, &path, entity)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a field list and add fields to the entity.
    fn process_field_list(
        &mut self,
        entity_id: usize,
        field_list: &FieldList,
    ) -> Result<(), String> {
        match field_list {
            FieldList::Field(field) => self.add_field_to_entity(entity_id, field),
            FieldList::C2(c2) => {
                self.process_field_list(entity_id, &c2.field_list)?;
                self.add_field_to_entity(entity_id, &c2.field)
            }
        }
    }

    /// Add a field to an entity in the model.
    fn add_field_to_entity(&mut self, entity_id: usize, field: &Field) -> Result<(), String> {
        let field_name = field.identifier.as_ref().trim().to_string();
        let location = self.location(field.location.as_ref());
        let field_value = Self::convert_field_value(&field.field_value)?;
        self.model
            .add_field_with_location(entity_id, &field_name, field_value, location)
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
