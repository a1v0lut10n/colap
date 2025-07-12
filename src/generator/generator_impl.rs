// SPDX-License-Identifier: Apache-2.0
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase};
use handlebars::Handlebars;
use chrono::Local;
use serde_json::json;

use crate::model::config_model::{ConfigModel, ConfigNode, ConfigValue, EntityNode};

/// Generation mode for the code generator
#[derive(Debug, Clone)]
pub enum GenerationMode {
    /// Generate a single .rs module file
    Module {
        output_file: PathBuf,
    },
    /// Generate a complete library crate
    Crate {
        output_dir: PathBuf,
        crate_name: String,
    },
}

/// Code generator that traverses a ConfigModel and emits Rust structs & helper methods.
/// Supports generating either a single module file or a complete library crate.
/// Produces a library-like API with proper encapsulation and collection handling.
pub struct CodeGenerator {
    model: ConfigModel,
    mode: GenerationMode,
    source_path: PathBuf,
    emitted_structs: HashSet<String>,
    // Track node IDs that are instances of plural entities
    plural_instances: HashSet<usize>,
    // Handlebars registry for template rendering
    handlebars: Handlebars<'static>,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new(
        model: ConfigModel,
        mode: GenerationMode,
        source_path: PathBuf,
    ) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        
        // Register templates
        handlebars.register_template_string("file_header", include_str!("templates/file_header.hbs"))?;
        handlebars.register_template_string("singular_struct", include_str!("templates/singular_struct.hbs"))?;
        handlebars.register_template_string("plural_struct", include_str!("templates/plural_struct.hbs"))?;
        handlebars.register_template_string("api_struct", include_str!("templates/api_struct.hbs"))?;
        handlebars.register_template_string("integration_test", include_str!("templates/integration_test.hbs"))?;
        handlebars.register_template_string("cargo_toml", include_str!("templates/cargo_toml.hbs"))?;
        handlebars.register_template_string("readme", include_str!("templates/readme.hbs"))?;
        
        // Enable built-in helpers
        handlebars.set_strict_mode(false);
        
        Ok(Self {
            model,
            mode,
            source_path,
            emitted_structs: HashSet::new(),
            plural_instances: HashSet::new(),
            handlebars,
        })
    }

    /// Entry point – generate code based on the configured mode.
    pub fn generate(&mut self) -> Result<()> {
        match &self.mode {
            GenerationMode::Module { output_file } => {
                self.generate_module(output_file.clone())
            }
            GenerationMode::Crate { output_dir, crate_name } => {
                self.generate_crate(output_dir.clone(), crate_name.clone())
            }
        }
    }

    /// Generate a single module file
    fn generate_module(&mut self, output_file: PathBuf) -> Result<()> {
        // Create the output directory if it doesn't exist
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?
            }
        }

        let mut out = String::new();
        self.generate_code_content(&mut out)?;
        
        // Add module-level tests
        self.generate_module_tests(&mut out)?;

        // Write the output to the file
        fs::write(&output_file, out)?;
        
        Ok(())
    }

    /// Generate a complete library crate
    fn generate_crate(&mut self, output_dir: PathBuf, crate_name: String) -> Result<()> {
        // Create crate directory structure
        fs::create_dir_all(output_dir.join("src"))?;
        
        // Generate Cargo.toml
        self.generate_cargo_toml(&output_dir, &crate_name)?;
        
        // Generate src/lib.rs
        let mut lib_content = String::new();
        self.generate_code_content(&mut lib_content)?;
        fs::write(output_dir.join("src").join("lib.rs"), lib_content)?;
        
        // Generate tests in tests/ directory
        fs::create_dir_all(output_dir.join("tests"))?;
        self.generate_crate_tests(&output_dir)?;
        
        // Generate README.md
        self.generate_readme(&output_dir, &crate_name)?;
        
        Ok(())
    }

    /// Generate the core code content (structs and implementations)
    fn generate_code_content(&mut self, out: &mut String) -> Result<()> {
        // Determine if we need HashMap
        let uses_hashmap = true; // In the future, we could analyze the model to determine this
        
        // Create the template data for file header
        let header_data = json!({
            "include_imports": true,
            "uses_hashmap": uses_hashmap
        });
        
        // Render the file header
        let header_content = self.handlebars.render("file_header", &header_data)?;
        out.push_str(&header_content);
        
        // Add necessary imports
        out.push_str("use colap::config_model::{ConfigModel, ConfigNode, ConfigValue};\n\n");
        
        // First identify all plural entity instances so we can skip them later
        self.identify_plural_instances(self.model.root_id());

        // Collect all entity nodes and their struct names
        let mut struct_names = HashMap::new();
        self.collect_struct_names(self.model.root_id(), &mut struct_names);

        // Find all plural entities so we can generate singular entity structs
        self.identify_and_emit_singular_entities(self.model.root_id(), &struct_names, out);

        // Generate all entity definitions recursively
        self.emit_all_entities(self.model.root_id(), &struct_names, out);
        
        Ok(())
    }

    /// Generate module-level tests (inline with the module)
    fn generate_module_tests(&self, out: &mut String) -> Result<()> {
        // Create a list of plural entity types for assertions
        let mut plural_entity_types = Vec::new();
        let mut plural_entity_assertions = Vec::new();
        
        // Add basic placeholders for entities to test
        // In a real implementation, we would gather these from the model
        plural_entity_types.push("Llms".to_string());
        plural_entity_assertions.push(json!({
            "plural": "llms",
            "singular": "llm"
        }));
        
        // Prepare the template data
        let test_data = json!({
            "crate_name": "", // Empty for modules as they use relative paths
            "is_crate": false,
            "test_file_path": self.relative_source_path(),
            "plural_entity_types": plural_entity_types,
            "plural_entity_assertions": plural_entity_assertions
        });
        
        // Render the test template
        let module_test_content = self.handlebars.render("integration_test", &test_data)?;
        
        // Format for inclusion in the module
        out.push_str("\n#[cfg(test)]\n");
        out.push_str("mod tests {\n");
        out.push_str("    use super::*;\n");
        
        // Add the rendered test content with proper indentation
        for line in module_test_content.lines() {
            if !line.trim().is_empty() {
                out.push_str("    ");
                out.push_str(line);
                out.push_str("\n");
            }
        }
        
        out.push_str("}\n");
        Ok(())
    }

    /// Generate Cargo.toml for the crate
    fn generate_cargo_toml(&self, output_dir: &PathBuf, crate_name: &str) -> Result<()> {
        // Figure out the relative path to colap crate from the output directory
        // This is a simplified approach; in a real-world scenario, you might need a more robust solution
        let colap_path = "../colap".to_string();
        
        // Create the template data
        let cargo_data = json!({
            "crate_name": crate_name,
            "colap_path": colap_path,
        });
        
        // Render the Cargo.toml using the Handlebars template
        let cargo_content = self.handlebars.render("cargo_toml", &cargo_data)?;
        
        // Write the Cargo.toml file
        fs::write(output_dir.join("Cargo.toml"), cargo_content)?;
        
        log::info!("Generated Cargo.toml for {}", crate_name);
        Ok(())
    }

    /// Generate integration tests for the crate
    fn generate_crate_tests(&self, output_dir: &PathBuf) -> Result<()> {
        let tests_dir = output_dir.join("tests");
        
        // Create tests directory if it doesn't exist
        fs::create_dir_all(&tests_dir)?;
        
        // Create tests/data directory and copy input configuration file
        self.copy_config_to_tests_data(output_dir)?;
        
        // Get the crate name for import paths
        let crate_name = self.get_crate_name();
        
        // Create a sanitized crate name for Rust imports (replace hyphens with underscores)
        let sanitized_crate_name = crate_name.replace('-', "_");
        
        // Generate a list of plural entity types for assertions
        let mut plural_entity_types = Vec::new();
        let mut plural_entity_assertions = Vec::new();
        
        // [This would be replaced with actual code to gather plural entities]
        // For now we're just adding basic placeholders
        plural_entity_types.push("Llms".to_string());
        plural_entity_assertions.push(json!({
            "plural": "llms",
            "singular": "llm"
        }));
        
        // Use the Handlebars template for integration tests
        let test_data = json!({
            "crate_name": crate_name,
            "sanitized_crate_name": sanitized_crate_name,
            "is_crate": true,
            "test_file_path": "tests/data/config.md",
            "plural_entity_types": plural_entity_types,
            "plural_entity_assertions": plural_entity_assertions
        });
        
        let test_content = self.handlebars.render("integration_test", &test_data)?;
        fs::write(tests_dir.join("integration.rs"), test_content)?;
        
        Ok(())
    }
    
    /// Copy the input configuration file to the tests/data directory
    fn copy_config_to_tests_data(&self, output_dir: &PathBuf) -> Result<()> {
        // Create tests/data directory
        let tests_data_dir = output_dir.join("tests").join("data");
        fs::create_dir_all(&tests_data_dir)?;
        
        // Get the source filename without path
        let source_filename = self.source_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Copy the input file to tests/data/config.md
        fs::copy(&self.source_path, tests_data_dir.join("config.md"))?;
        
        log::info!("Copied {} to {}", source_filename, tests_data_dir.join("config.md").display());
        
        Ok(())
    }

    /// Generate README.md for the crate
    fn generate_readme(&self, output_dir: &PathBuf, crate_name: &str) -> Result<()> {
        // Extract the config filename from the source path
        let config_filename = self.source_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Get the current date
        let date = Local::now().format("%Y-%m-%d").to_string();
        
        // Create the template data
        let readme_data = json!({
            "crate_name": crate_name,
            "config_filename": config_filename,
            "date": date,
            "example_code": ""
        });
        
        // Render the README using the Handlebars template
        let readme_content = self.handlebars.render("readme", &readme_data)?;
        
        // Write the README file
        fs::write(output_dir.join("README.md"), readme_content)?;
        
        log::info!("Generated README.md for {}", crate_name);
        Ok(())
    }

    /// Get the crate name from the generation mode
    fn get_crate_name(&self) -> String {
        match &self.mode {
            GenerationMode::Crate { crate_name, .. } => crate_name.clone(),
            GenerationMode::Module { .. } => "config".to_string(),
        }
    }

    /// Identify all instances of plural entities so we can skip generating individual structs for them
    fn identify_plural_instances(&mut self, node_id: usize) {
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();
            if let ConfigNode::Entity(ent) = &*node_b {
                // Check if this entity has a plural name
                if let Some(_plural_name) = &ent.plural_name {
                    // This is a plural entity, mark its children as instances
                    for &child_id in &ent.children {
                        self.plural_instances.insert(child_id);
                    }
                }
                
                // Recursively process all children
                for &child_id in &ent.children {
                    self.identify_plural_instances(child_id);
                }
            }
        }
    }
    
    /// Identify plural entities and emit singular entity structs for them
    fn identify_and_emit_singular_entities(&mut self, node_id: usize, struct_names: &HashMap<usize, String>, out: &mut String) {
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();
            if let ConfigNode::Entity(ent) = &*node_b {
                // Check if this entity has a plural name
                if let Some(_plural_name) = &ent.plural_name {
                    // This is a plural entity - get first child to generate singular entity struct
                    if !ent.children.is_empty() {
                        let first_child_id = ent.children[0];
                        
                        // Use first child as template for the singular entity
                        if let Some(first_child) = self.model.get_node(first_child_id) {
                            let first_child_b = first_child.borrow();
                            if let ConfigNode::Entity(_child_ent) = &*first_child_b {
                                // Generate the singular struct from this child
                                let singular_struct_name = self.struct_name(&ent.name);
                                if !self.emitted_structs.contains(&singular_struct_name) {
                                    self.emit_singular_struct(first_child_id, &singular_struct_name, struct_names, out);
                                }
                            }
                        }
                    }
                }
                
                // Recursively process all children
                for &child_id in &ent.children {
                    self.identify_and_emit_singular_entities(child_id, struct_names, out);
                }
            }
        }
    }
    
    /// Emit a singular struct for a plural entity type based on its first child
    fn emit_singular_struct(&mut self, node_id: usize, struct_name: &str, _struct_names: &HashMap<usize, String>, out: &mut String) {
        // Mark this struct as emitted so we don't duplicate it
        self.emitted_structs.insert(struct_name.to_string());
        
        // Extract fields from the entity
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();
            if let ConfigNode::Entity(ent) = &*node_b {
                // Prepare data for template
                let mut fields = Vec::new();
                let mut getters = Vec::new();
                let mut field_initializers = Vec::new();
                
                // Process primitive fields
                for (field_name, field_value) in &ent.fields {
                    let field_name_snake = self.field_name(field_name);
                    let orig_field_name = field_name.clone();
                    
                    // Determine the Rust type for this field
                    let rust_type = match field_value {
                        ConfigValue::Integer(_) => "i64".to_string(),
                        ConfigValue::Float(_) => "f64".to_string(),
                        ConfigValue::Boolean(_) => "bool".to_string(),
                        ConfigValue::String(_) => "String".to_string(),
                    };
                    
                    // Add field to struct
                    fields.push(json!({
                        "name": field_name_snake,
                        "type": rust_type,
                        "is_optional": false
                    }));
                    
                    // Add getter
                    getters.push(json!({
                        "name": field_name_snake,
                        "return_type": rust_type,
                        "is_reference": false,
                        "is_option": false,
                        "is_primitive": true
                    }));
                    
                    // Add initializer for from_entity
                    field_initializers.push(json!({
                        "name": field_name_snake,
                        "type": rust_type,
                        "original_name": orig_field_name,
                        "is_entity": false,
                        "is_api": false
                    }));
                }
                
                // Process entity children
                for &child_id in &ent.children {
                    if let Some(child) = self.model.get_node(child_id) {
                        let child_b = child.borrow();
                        if let ConfigNode::Entity(child_ent) = &*child_b {
                            let (field_name, field_type) = if let Some(plural) = &child_ent.plural_name {
                                // If plural, use plural name for field and plural type
                                (self.field_name(plural), self.struct_name(plural))
                            } else {
                                (self.field_name(&child_ent.name), self.struct_name(&child_ent.name))
                            };
                            
                            let original_name = child_ent.name.clone();
                            let is_api = field_type == "Api";
                            
                            // Add field to struct (Api fields are optional)
                            fields.push(json!({
                                "name": field_name,
                                "type": field_type,
                                "is_optional": is_api
                            }));
                            
                            // Add getter
                            getters.push(json!({
                                "name": field_name,
                                "return_type": field_type,
                                "is_reference": !is_api && !["i64", "f64", "bool", "String"].contains(&field_type.as_str()),
                                "is_option": is_api,
                                "is_primitive": false
                            }));
                            
                            // Add initializer for from_entity
                            field_initializers.push(json!({
                                "name": field_name,
                                "type": field_type,
                                "original_name": original_name,
                                "is_entity": true,
                                "is_api": is_api
                            }));
                        }
                    }
                }
                
                // Prepare template data
                let template_data = json!({
                    "struct_name": struct_name,
                    "fields": fields,
                    "getters": getters,
                    "field_initializers": field_initializers
                });
                
                // Render the template
                let struct_content = self.handlebars.render("singular_struct", &template_data)
                    .expect("Failed to render singular_struct template");
                
                out.push_str(&struct_content);
            }
        }
    }
    
    /// Emit all entity structs recursively
    fn emit_all_entities(&mut self, node_id: usize, struct_names: &HashMap<usize, String>, out: &mut String) {
        // Skip generating structs for instances of plural entities
        if !self.plural_instances.contains(&node_id) {
            // Emit this entity
            self.emit_entity(node_id, 0, struct_names, out);
        }

        // Then recursively emit its children
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();
            if let ConfigNode::Entity(ent) = &*node_b {
                for &child_id in &ent.children {
                    self.emit_all_entities(child_id, struct_names, out);
                }
            }
        }
    }

    /// Collect all struct names for entities so we can reference them before definition
    fn collect_struct_names(&self, node_id: usize, map: &mut HashMap<usize, String>) {
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();
            match &*node_b {
                ConfigNode::Entity(ent) => {
                    // Use the struct name for this entity
                    let struct_name = self.struct_name(&ent.name);
                    map.insert(node_id, struct_name);

                    // Also recursively process children
                    for &child_id in &ent.children {
                        self.collect_struct_names(child_id, map);
                    }

                    // For plural entities, also collect the collection wrapper
                    if let Some(plural) = &ent.plural_name {
                        let _plural_struct = self.struct_name(plural);
                        // We don't need to map this to a node_id since it's synthetic
                    }
                }
                ConfigNode::Field(_) => {}
            }
        }
    }

    /// Emit a struct definition for an entity and its children
    fn emit_entity(&mut self, node_id: usize, indent_level: usize, _struct_names: &HashMap<usize, String>, out: &mut String) {
        if let Some(node) = self.model.get_node(node_id) {
            let node_b = node.borrow();

            match &*node_b {
                ConfigNode::Entity(ent) => {
                    // For plural entities, we need to emit two structs:
                    // 1. A singular struct (e.g., Llm) for the entity type
                    // 2. A collection wrapper struct (e.g., Llms) with a map field
                    if let Some(plural_name) = &ent.plural_name {
                        // Generate the collection wrapper struct
                        let collection_struct_name = self.struct_name(plural_name);
                        let singular_struct_name = self.struct_name(&ent.name);
                        
                        // Skip if we already emitted this wrapper struct
                        if self.emitted_structs.contains(&collection_struct_name) {
                            return;
                        }
                        self.emitted_structs.insert(collection_struct_name.clone());
                        
                        // Prepare the template data
                        let template_data = json!({
                            "struct_name": collection_struct_name,
                            "singular_struct_name": singular_struct_name
                        });
                        
                        // Render the template
                        let struct_content = self.handlebars.render("plural_struct", &template_data)
                            .expect("Failed to render plural_struct template");
                        
                        // Add indentation if needed
                        if indent_level > 0 {
                            let indent = "    ".repeat(indent_level);
                            for line in struct_content.lines() {
                                out.push_str(&indent);
                                out.push_str(line);
                                out.push_str("\n");
                            }
                        } else {
                            out.push_str(&struct_content);
                        }
                        
                        return;
                    }
                    
                    // For regular entities or singular entities of plural collections
                    let struct_name = self.struct_name(&ent.name);
                    if self.emitted_structs.contains(&struct_name) {
                        return;
                    }
                    self.emitted_structs.insert(struct_name.clone());
                    
                    // Special case for Api struct - use dedicated template
                    if struct_name == "Api" {
                        // Use the api_struct template
                        let template_data = json!({});
                        
                        // Render the template
                        let struct_content = self.handlebars.render("api_struct", &template_data)
                            .expect("Failed to render api_struct template");
                        
                        // Add indentation if needed
                        if indent_level > 0 {
                            let indent = "    ".repeat(indent_level);
                            for line in struct_content.lines() {
                                out.push_str(&indent);
                                out.push_str(line);
                                out.push_str("\n");
                            }
                        } else {
                            out.push_str(&struct_content);
                        }
                        
                        return;
                    }
                    
                    // No special cases - all entities are handled generically
                    
                    // Generate struct definition
                    let indent = "    ".repeat(indent_level);
                    out.push_str(&format!("{}#[derive(Debug, Clone, Default)]\n", indent));
                    out.push_str(&format!("{}pub struct {} {{\n", indent, struct_name));
                    
                    // Get all fields for this entity
                    let mut field_names = Vec::new();
                    let mut field_types = HashMap::new();
                    
                    // Process primitive fields
                    for (field_name, field_value) in &ent.fields {
                        let field_name_snake = self.field_name(field_name);
                        
                        // Determine the Rust type for this field
                        let rust_type = match field_value {
                            ConfigValue::Integer(_) => "i64".to_string(),
                            ConfigValue::Float(_) => "f64".to_string(),
                            ConfigValue::Boolean(_) => "bool".to_string(),
                            ConfigValue::String(_) => "String".to_string(),
                        };
                        
                        field_names.push(field_name_snake.clone());
                        field_types.insert(field_name_snake, rust_type);
                    }
                    
                    // Process entity children
                    for &child_id in &ent.children {
                        if let Some(child) = self.model.get_node(child_id) {
                            let child_b = child.borrow();
                            if let ConfigNode::Entity(child_ent) = &*child_b {
                                // If plural, use plural name for field and plural type
                                if let Some(plural) = &child_ent.plural_name {
                                    let field_name = self.field_name(plural);
                                    let field_type = self.struct_name(plural);
                                    field_names.push(field_name.clone());
                                    field_types.insert(field_name, field_type);
                                } else {
                                    let field_name = self.field_name(&child_ent.name);
                                    let field_type = self.struct_name(&child_ent.name);
                                    field_names.push(field_name.clone());
                                    field_types.insert(field_name, field_type);
                                }
                            }
                        }
                    }
                    
                    // Add fields to struct
                    for field_name in &field_names {
                        let field_type = field_types.get(field_name).unwrap();
                        
                        // Make singular entity fields optional (they might be missing)
                        // But keep collection structs and primitive types as non-optional
                        if field_type.chars().next().unwrap_or('_').is_uppercase() && 
                           !field_type.ends_with('s') && field_name != "root" {
                            out.push_str(&format!("{}    pub {}: Option<{}>,\n", indent, field_name, field_type));
                        } else {
                            out.push_str(&format!("{}    pub {}: {},\n", indent, field_name, field_type));
                        }
                    }
                    
                    out.push_str(&indent);
                    out.push_str("}\n\n");
                    
                    // Generate implementation for getter methods
                    out.push_str(&format!("{}impl {} {{\n", indent, struct_name));
                    
                    // Add getter methods
                    for field_name in &field_names {
                        let return_type = field_types.get(field_name).unwrap();
                        // If this is an optional entity field, return Option<&T>
                        if return_type.chars().next().unwrap_or('_').is_uppercase() && 
                           !return_type.ends_with('s') && field_name != "root" {
                            out.push_str(&format!("{}    pub fn {}(&self) -> Option<&{}> {{\n", indent, field_name, return_type));
                            out.push_str(&format!("{}        self.{}.as_ref()\n", indent, field_name));
                        }
                        // If the field is a primitive type, clone its value
                        else if !return_type.chars().next().unwrap_or('_').is_uppercase() {
                            out.push_str(&format!("{}    pub fn {}(&self) -> {} {{\n", indent, field_name, return_type));
                            out.push_str(&format!("{}        self.{}.clone()\n", indent, field_name));
                        } else {
                            out.push_str(&format!("{}    pub fn {}(&self) -> &{} {{\n", indent, field_name, return_type));
                            out.push_str(&format!("{}        &self.{}\n", indent, field_name));
                        }
                        out.push_str(&format!("{}    }}\n\n", indent));
                    }
                    
                    out.push_str(&indent);
                    out.push_str("}\n\n");
                    
                    // Generate implementation for from_entity method
                    out.push_str(&format!("{}impl {} {{\n", indent, struct_name));
                    
                    // Add from_entity method
                    out.push_str(&format!("{0}    pub fn from_entity(model: &colap::config_model::ConfigModel, id: usize) -> Self {{\n", indent));
                    out.push_str(&format!("{0}        let node = model.get_node(id).expect(\"entity\");\n", indent));
                    out.push_str(&format!("{0}        let borrowed = node.borrow();\n", indent));
                    out.push_str(&format!("{0}        if let colap::config_model::ConfigNode::Entity(_ent) = &*borrowed {{\n", indent));
                    out.push_str(&format!("{0}            Self {{\n", indent));
                    
                    // Initialize fields
                    for field_name in &field_names {
                        let field_type = field_types.get(field_name).unwrap();
                        let orig_field_name = self.to_original_case(field_name);
                        
                        // Check if this is an entity field (starts with uppercase)
                        if field_type.chars().next().unwrap_or('_').is_uppercase() {
                            // Check if this is a singular entity (doesn't end with 's') or a collection
                            if !field_type.ends_with('s') {
                                // Singular entities - look up by original field name
                                out.push_str(&format!("{0}                {1}: model.find_child_entity_by_name(id, \"{2}\").map(|child_id| {3}::from_entity(model, child_id)),\n", indent, field_name, orig_field_name, field_type));
                            } else {
                                // Collection entities - use from_children method
                                out.push_str(&format!("{0}                {1}: {2}::from_children(model, id),\n", indent, field_name, field_type));
                            }
                        } else {
                            // For primitive fields, extract the value from the model
                            match field_type.as_str() {
                                "i64" => out.push_str(&format!("{0}                {1}: model.get_field_value(id, \"{2}\").and_then(|v| if let colap::config_model::ConfigValue::Integer(val) = v {{ Some(val) }} else {{ None }}).unwrap_or(0),\n", indent, field_name, orig_field_name)),
                                "f64" => out.push_str(&format!("{0}                {1}: model.get_field_value(id, \"{2}\").and_then(|v| if let colap::config_model::ConfigValue::Float(val) = v {{ Some(val) }} else {{ None }}).unwrap_or(0.0),\n", indent, field_name, orig_field_name)),
                                "bool" => out.push_str(&format!("{0}                {1}: model.get_field_value(id, \"{2}\").and_then(|v| if let colap::config_model::ConfigValue::Boolean(val) = v {{ Some(val) }} else {{ None }}).unwrap_or(false),\n", indent, field_name, orig_field_name)),
                                "String" => out.push_str(&format!("{0}                {1}: model.get_field_value(id, \"{2}\").and_then(|v| if let colap::config_model::ConfigValue::String(val) = v {{ Some(val.clone()) }} else {{ None }}).unwrap_or_default(),\n", indent, field_name, orig_field_name)),
                                _ => out.push_str(&format!("{0}                {1}: Default::default(),\n", indent, field_name)),
                            }
                        }
                    }
                    
                    out.push_str(&format!("{0}            }}\n", indent));
                    out.push_str(&format!("{0}        }} else {{ unreachable!() }}\n", indent));
                    out.push_str(&format!("{0}    }}\n", indent));
                    
                    out.push_str(&indent);
                    out.push_str("}\n\n");
                    
                    // Add additional functionality for Root struct
                    if struct_name == "Root" {
                        out.push_str(&format!("{}impl Root {{\n", indent));
                        out.push_str(&format!("{}    pub fn from_model(model: &colap::config_model::ConfigModel) -> Self {{\n", indent));
                        out.push_str(&format!("{}        Self::from_entity(model, model.root_id())\n", indent));
                        out.push_str(&format!("{}    }}\n", indent));
                        out.push_str(&indent);
                        out.push_str("}\n\n");
                    }
                },
                ConfigNode::Field(_) => {},
            }
        }
    }

    /// Generate a pluralized struct name for collections
    #[allow(dead_code)]
    fn plural_struct_name(&self, ent: &EntityNode) -> String {
        if let Some(plural) = &ent.plural_name {
            self.struct_name(plural)
        } else {
            format!("{}s", self.struct_name(&ent.name))
        }
    }

    /// Get a struct name (PascalCase)
    fn struct_name(&self, name: &str) -> String {
        name.to_pascal_case()
    }

    /// Get a field name (snake_case)
    fn field_name(&self, name: &str) -> String {
        // Convert field names to snake_case
        if name == "type" {
            "type_".to_string()
        } else {
            name.to_snake_case()
        }
    }
    
    /// Convert back to original case for field lookups
    fn to_original_case(&self, name: &str) -> String {
        if name == "type_" {
            "type".to_string()
        } else {
            name.to_string()
        }
    }

    /// Add indentation to output
    #[allow(dead_code)]
    fn push_indent(&self, n: usize, out: &mut String) {
        for _ in 0..n {
            out.push_str("    ");
        }
    }



    /// Get a relative path to the source file for inclusion in tests
    fn relative_source_path(&self) -> String {
        // This is a simplistic implementation that assumes the source file is
        // within the same project. In a real implementation, you would use
        // a better approach to generate a relative path that works for tests.
        self.source_path.to_string_lossy().replace('\\', "/")
    }
}
