// SPDX-License-Identifier: Apache-2.0
use crate::cola_actions::{
    CodeBlock, Cola, Entity, FieldList, FieldValue, MarkdownItem, NestedBlock,
};
use crate::config_model::{ConfigModel, ConfigValue};
use crate::source_location::SourceLocation;
use std::path::PathBuf;

/// Builds a ConfigModel from a parsed Cola AST
pub struct ModelBuilder;

impl ModelBuilder {
    /// Convert a Cola AST to a ConfigModel
    pub fn build_config_model(cola: &Cola) -> Result<ConfigModel, String> {
        let mut model = ConfigModel::new();
        let root_id = model.root_id();

        if let Some(markdown_items) = cola {
            for markdown_item in markdown_items {
                match markdown_item {
                    MarkdownItem::CodeBlock(CodeBlock::ColaCodeBlock(cola_block)) => {
                        if let Some(entities) = &cola_block.cola_syntax {
                            for entity in entities {
                                Self::process_entity(&mut model, root_id, "", entity)?;
                            }
                        }
                    }
                    _ => {} // Ignore non-cola code blocks, headings, paragraphs
                }
            }
        }

        Ok(model)
    }

    /// Process an entity and add it to the ConfigModel
    fn process_entity(
        model: &mut ConfigModel,
        _parent_id: usize,
        parent_path: &str,
        entity: &Entity,
    ) -> Result<(), String> {
        match entity {
            Entity::SingularEntity(singular) => {
                // Create entity path - get the identifier string
                // We need to handle this differently since we can't directly access ValLoc fields
                let identifier = &singular.identifier;
                // Extract the string value
                let entity_name = match identifier.as_ref() {
                    s => s.trim(), // Trim the string
                };
                let path = if parent_path.is_empty() {
                    entity_name.to_string()
                } else {
                    format!("{}/{}", parent_path, entity_name)
                };

                // Extract source location from the rustemo ValLoc object
                let location = singular.location.as_ref().map(|loc| {
                    // Convert rustemo Location to our SourceLocation
                    // Extract start position (line, column)
                    let (start_line, start_column) = match &loc.start {
                        rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                        rustemo::Position::Position(_) => (1, 0), // Fallback for byte offset position
                    };
                    
                    // Extract end position (line, column) if available
                    let (end_line, end_column) = if let Some(end) = &loc.end {
                        match end {
                            rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                            rustemo::Position::Position(_) => (start_line, start_column), // Fallback
                        }
                    } else {
                        (start_line, start_column) // Default to start position if end is not available
                    };
                    
                    SourceLocation {
                        file_path: PathBuf::new(), // We may not have a file path in the Location
                        start_line: start_line as u32,
                        start_column: start_column as u32,
                        end_line: end_line as u32,
                        end_column: end_column as u32,
                    }
                });

                // Create the entity at this path
                let entity_id =
                    model.create_entity_at_path(parent_path, entity_name, None, location)?;

                // Process entity contents
                Self::process_entity_definition(
                    model,
                    entity_id,
                    &path,
                    &singular.entity_definition,
                )?;

                Ok(())
            }
            Entity::PluralEntity(plural) => {
                // Create entity path - extract the identifiers
                let id1 = &plural.identifier_1;
                let id3 = &plural.identifier_3;
                // Extract string values
                let entity_name = match id1.as_ref() {
                    s => s.trim(),
                };
                let plural_name = match id3.as_ref() {
                    s => s.trim(),
                };
                let path = if parent_path.is_empty() {
                    entity_name.to_string()
                } else {
                    format!("{}/{}", parent_path, entity_name)
                };

                // Extract source location from the rustemo ValLoc object
                let location = plural.location.as_ref().map(|loc| {
                    // Convert rustemo Location to our SourceLocation
                    // Extract start position (line, column)
                    let (start_line, start_column) = match &loc.start {
                        rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                        rustemo::Position::Position(_) => (1, 0), // Fallback for byte offset position
                    };
                    
                    // Extract end position (line, column) if available
                    let (end_line, end_column) = if let Some(end) = &loc.end {
                        match end {
                            rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                            rustemo::Position::Position(_) => (start_line, start_column), // Fallback
                        }
                    } else {
                        (start_line, start_column) // Default to start position if end is not available
                    };
                    
                    SourceLocation {
                        file_path: PathBuf::new(), // We may not have a file path in the Location
                        start_line: start_line as u32,
                        start_column: start_column as u32,
                        end_line: end_line as u32,
                        end_column: end_column as u32,
                    }
                });

                // Create the entity at this path with plural name
                let entity_id = model.create_entity_at_path(
                    parent_path,
                    entity_name,
                    Some(plural_name),
                    location,
                )?;

                // Process entity contents
                Self::process_entity_definition(
                    model,
                    entity_id,
                    &path,
                    &plural.entity_definition,
                )?;

                Ok(())
            }
        }
    }

    /// Process the contents of an entity definition
    fn process_entity_definition(
        model: &mut ConfigModel,
        entity_id: usize,
        entity_path: &str,
        entity_def: &Option<Vec<NestedBlock>>,
    ) -> Result<(), String> {
        if let Some(nested_blocks) = entity_def {
            for nested_block in nested_blocks {
                match nested_block {
                    NestedBlock::FieldList(field_list) => {
                        Self::process_field_list(model, entity_id, field_list)?;
                    }
                    NestedBlock::Entity(entity) => {
                        Self::process_entity(model, entity_id, entity_path, entity)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a field list and add fields to the entity
    fn process_field_list(
        model: &mut ConfigModel,
        entity_id: usize,
        field_list: &FieldList,
    ) -> Result<(), String> {
        match field_list {
            FieldList::Field(field) => {
                Self::add_field_to_entity(model, entity_id, field)?;
            }
            FieldList::C2(field_list_c2) => {
                Self::process_field_list(model, entity_id, &field_list_c2.field_list)?;
                Self::add_field_to_entity(model, entity_id, &field_list_c2.field)?;
            }
        }

        Ok(())
    }

    /// Add a field to an entity in the model
    fn add_field_to_entity(
        model: &mut ConfigModel,
        entity_id: usize,
        field: &crate::cola_actions::Field,
    ) -> Result<(), String> {
        // Extract field name from identifier
        let id = &field.identifier;
        let field_name = match id.as_ref() {
            s => s.trim().to_string(),
        };
        
        // Extract source location from the field
        let location = field.location.as_ref().map(|loc| {
            // Convert rustemo Location to our SourceLocation
            // Extract start position (line, column)
            let (start_line, start_column) = match &loc.start {
                rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                rustemo::Position::Position(_) => (1, 0), // Fallback for byte offset position
            };
            
            // Extract end position (line, column) if available
            let (end_line, end_column) = if let Some(end) = &loc.end {
                match end {
                    rustemo::Position::LineBased(lc) => (lc.line, lc.column),
                    rustemo::Position::Position(_) => (start_line, start_column), // Fallback
                }
            } else {
                (start_line, start_column) // Default to start position if end is not available
            };
            
            SourceLocation {
                file_path: PathBuf::new(), // We may not have a file path in the Location
                start_line: start_line as u32,
                start_column: start_column as u32,
                end_line: end_line as u32,
                end_column: end_column as u32,
            }
        });
        
        // Pass field_value to be converted
        let field_value = Self::convert_field_value(&field.field_value)?;
        
        // Add field with source location to the entity
        model.add_field_with_location(entity_id, &field_name, field_value, location)?;
        
        Ok(())
    }

    /// Convert a FieldValue from the AST to a ConfigValue for the model
    fn convert_field_value(field_value: &FieldValue) -> Result<ConfigValue, String> {
        match field_value {
            FieldValue::QuotedStringDouble(s) => {
                // Extract string and remove surrounding quotes
                let s_val = match s.as_ref() {
                    s => s.trim(),
                };
                let content = s_val[1..s_val.len() - 1].to_string();
                Ok(ConfigValue::String(content))
            }
            FieldValue::QuotedStringSingle(s) => {
                // Extract string and remove surrounding quotes
                let s_val = match s.as_ref() {
                    s => s.trim(),
                };
                let content = s_val[1..s_val.len() - 1].to_string();
                Ok(ConfigValue::String(content))
            }
            FieldValue::Number(n) => {
                let n_str = match n.as_ref() {
                    s => s.trim(),
                };
                if n_str.contains('.') {
                    // Float value
                    match n_str.parse::<f64>() {
                        Ok(f) => Ok(ConfigValue::Float(f)),
                        Err(_) => Err(format!("Failed to parse float: {}", n_str)),
                    }
                } else {
                    // Integer value
                    match n_str.parse::<i64>() {
                        Ok(i) => Ok(ConfigValue::Integer(i)),
                        Err(_) => Err(format!("Failed to parse integer: {}", n_str)),
                    }
                }
            }
            FieldValue::BooleanTrue => Ok(ConfigValue::Boolean(true)),
            FieldValue::BooleanFalse => Ok(ConfigValue::Boolean(false)),
        }
    }
}
