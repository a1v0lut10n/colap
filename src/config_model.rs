// SPDX-License-Identifier: Apache-2.0
use crate::source_location::SourceLocation;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub type NodeId = usize;
pub type NodeRef = Rc<RefCell<ConfigNode>>;

/// Represents the different types of values a configuration field can have
#[derive(Debug, Clone)]
pub enum ConfigValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

/// Represents a field in a configuration entity
#[derive(Debug)]
pub struct ConfigField {
    pub name: String,
    pub value: ConfigValue,
    pub location: Option<SourceLocation>,
}

/// Represents an entity in the configuration
#[derive(Debug)]
pub struct EntityNode {
    pub name: String,                         // Singular form name
    pub plural_name: Option<String>,          // Optional plural form
    pub parent: Option<NodeId>,               // Parent entity ID
    pub children: Vec<NodeId>,                // Child entity IDs
    pub fields: HashMap<String, ConfigValue>, // Field name to value mapping
    pub location: Option<SourceLocation>,     // Source location
}

/// The main node type for our configuration model
#[derive(Debug)]
pub enum ConfigNode {
    Entity(EntityNode),
    Field(ConfigField),
}

impl ConfigNode {
    /// Create a new entity node
    pub fn new_entity(
        name: &str,
        plural_name: Option<&str>,
        parent: Option<NodeId>,
        location: Option<SourceLocation>,
    ) -> Self {
        ConfigNode::Entity(EntityNode {
            name: name.to_string(),
            plural_name: plural_name.map(|s| s.to_string()),
            parent,
            children: vec![],
            fields: HashMap::new(),
            location,
        })
    }

    /// Create a new field node
    pub fn new_field(name: &str, value: ConfigValue, location: Option<SourceLocation>) -> Self {
        ConfigNode::Field(ConfigField {
            name: name.to_string(),
            value,
            location,
        })
    }

    /// Get the name of this node
    pub fn name(&self) -> &str {
        match self {
            ConfigNode::Entity(entity) => &entity.name,
            ConfigNode::Field(field) => &field.name,
        }
    }

    /// Check if this node is an entity
    pub fn is_entity(&self) -> bool {
        matches!(self, ConfigNode::Entity(_))
    }

    /// Check if this node is a field
    pub fn is_field(&self) -> bool {
        matches!(self, ConfigNode::Field(_))
    }
}

/// The model that holds the entire configuration structure
#[derive(Debug)]
pub struct ConfigModel {
    nodes: HashMap<NodeId, NodeRef>,
    root_id: NodeId,
    original_entity_names: HashMap<String, String>, // Added to store original quoted entity names
}

impl ConfigModel {
    /// Create a new empty model
    pub fn new() -> Self {
        let mut model = ConfigModel {
            nodes: HashMap::new(),
            root_id: 0,
            original_entity_names: HashMap::new(),
        };

        // Create and set the root node
        let root_node = ConfigNode::new_entity("root", None, None, None);
        let root_id = model.add_node(root_node);
        model.root_id = root_id;
        model
    }

    /// Set original entity names mapping
    pub fn set_original_entity_names(&mut self, names: HashMap<String, String>) {
        self.original_entity_names = names;
    }

    /// Get original entity name if it exists, otherwise return the sanitized name
    pub fn get_original_entity_name(&self, sanitized_name: &str) -> String {
        if let Some(original) = self.original_entity_names.get(sanitized_name) {
            original.clone()
        } else {
            sanitized_name.to_string()
        }
    }

    /// Add a node to the model and return its ID
    pub fn add_node(&mut self, node: ConfigNode) -> NodeId {
        let id = self.nodes.len();
        self.nodes.insert(id, Rc::new(RefCell::new(node)));
        id
    }

    /// Get a node by its ID
    pub fn get_node(&self, id: NodeId) -> Option<NodeRef> {
        self.nodes.get(&id).cloned()
    }

    /// Get the root node ID
    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    /// Add a child node to a parent
    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), String> {
        // Get the parent and child nodes
        let parent_node = self
            .get_node(parent_id)
            .ok_or_else(|| format!("Parent node with ID {} not found", parent_id))?;

        let child_node = self
            .get_node(child_id)
            .ok_or_else(|| format!("Child node with ID {} not found", child_id))?;

        // Add the child ID to the parent's children
        {
            let mut parent_node_borrow = parent_node.borrow_mut();
            if let ConfigNode::Entity(ref mut entity) = *parent_node_borrow {
                entity.children.push(child_id);
            } else {
                return Err(format!(
                    "Parent node with ID {} is not an entity",
                    parent_id
                ));
            }
        }

        // Set the parent ID on the child node
        {
            let mut child_node_borrow = child_node.borrow_mut();
            if let ConfigNode::Entity(ref mut entity) = *child_node_borrow {
                entity.parent = Some(parent_id);
            }
            // Note: Fields don't track their parent as they're owned by entities directly
        }

        Ok(())
    }

    /// Add a field to an entity
    pub fn add_field_to_entity(
        &mut self,
        entity_id: NodeId,
        field_name: &str,
        value: ConfigValue,
    ) -> Result<(), String> {
        let entity_node = self
            .get_node(entity_id)
            .ok_or_else(|| format!("Entity node with ID {} not found", entity_id))?;

        let mut entity_node_borrow = entity_node.borrow_mut();
        if let ConfigNode::Entity(ref mut entity) = *entity_node_borrow {
            entity.fields.insert(field_name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Node with ID {} is not an entity", entity_id))
        }
    }
    
    /// Add a field to an entity with source location
    pub fn add_field_with_location(
        &mut self,
        entity_id: NodeId,
        field_name: &str,
        value: ConfigValue,
        location: Option<SourceLocation>,
    ) -> Result<(), String> {
        // First, add the field value to the entity's fields map for direct lookup
        self.add_field_to_entity(entity_id, field_name, value.clone())?;
        
        // Create a field node with the location
        let field_node = ConfigNode::new_field(field_name, value, location);
        
        // Add the field node to the model
        let field_id = self.add_node(field_node);
        
        // Add the field as a child of the entity
        self.add_child(entity_id, field_id)
    }

    /// Find an entity by path (e.g., "llm/openai")
    pub fn find_entity_by_path(&self, path: &str) -> Option<NodeId> {
        if path.is_empty() {
            return Some(self.root_id);
        }

        let components: Vec<&str> = path.split('/').collect();
        let mut current_id = self.root_id;

        for component in components {
            let found = {
                let current_node = self.get_node(current_id)?;
                let current_node_borrow = current_node.borrow();

                if let ConfigNode::Entity(entity) = &*current_node_borrow {
                    let child_ids = &entity.children;

                    // Find a child entity with the matching name
                    child_ids
                        .iter()
                        .find(|&&id| {
                            if let Some(child_node) = self.get_node(id) {
                                let child_borrow = child_node.borrow();
                                if let ConfigNode::Entity(child_entity) = &*child_borrow {
                                    return child_entity.name == component;
                                }
                            }
                            false
                        })
                        .cloned()
                } else {
                    None
                }
            };

            if let Some(next_id) = found {
                current_id = next_id;
            } else {
                return None;
            }
        }

        Some(current_id)
    }

    /// Create an entity at the specified path
    pub fn create_entity_at_path(
        &mut self,
        path: &str,
        name: &str,
        plural_name: Option<&str>,
        location: Option<SourceLocation>,
    ) -> Result<NodeId, String> {
        // If path is empty, create at root
        if path.is_empty() {
            let entity = ConfigNode::new_entity(name, plural_name, Some(self.root_id), location);
            let entity_id = self.add_node(entity);
            self.add_child(self.root_id, entity_id)?;
            return Ok(entity_id);
        }

        // Find the parent entity
        let parent_id = self
            .find_entity_by_path(path)
            .ok_or_else(|| format!("Parent path '{}' not found", path))?;

        // Create the new entity
        let entity = ConfigNode::new_entity(name, plural_name, Some(parent_id), location);
        let entity_id = self.add_node(entity);

        // Add it as a child of the parent
        self.add_child(parent_id, entity_id)?;

        Ok(entity_id)
    }

    /// Get a field value from an entity
    pub fn get_field_value(&self, entity_id: NodeId, field_name: &str) -> Option<ConfigValue> {
        let entity_node = self.get_node(entity_id)?;
        let entity_borrow = entity_node.borrow();

        if let ConfigNode::Entity(entity) = &*entity_borrow {
            entity.fields.get(field_name).cloned()
        } else {
            None
        }
    }

    /// Find a child entity by name within a parent entity
    pub fn find_child_entity_by_name(&self, parent_id: NodeId, child_name: &str) -> Option<NodeId> {
        let parent_node = self.get_node(parent_id)?;
        let parent_borrow = parent_node.borrow();
        
        if let ConfigNode::Entity(parent_entity) = &*parent_borrow {
            for &child_id in &parent_entity.children {
                if let Some(child_node) = self.get_node(child_id) {
                    let child_borrow = child_node.borrow();
                    if let ConfigNode::Entity(child_entity) = &*child_borrow {
                        if child_entity.name == child_name {
                            return Some(child_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Display the node tree recursively
    fn display_node(&self, id: NodeId, depth: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let node = match self.get_node(id) {
            Some(n) => n,
            None => return writeln!(f, "{}Node ID {} not found", "  ".repeat(depth), id),
        };

        let node_borrow = node.borrow();
        match &*node_borrow {
            ConfigNode::Entity(entity) => {
                // Print entity name (and plural if available)
                if let Some(plural) = &entity.plural_name {
                    writeln!(
                        f,
                        "{}{} plural {}:",
                        "  ".repeat(depth),
                        self.get_original_entity_name(&entity.name),
                        plural
                    )?;
                } else {
                    writeln!(
                        f,
                        "{}{}:",
                        "  ".repeat(depth),
                        self.get_original_entity_name(&entity.name)
                    )?;
                }

                // Print fields
                for (name, value) in &entity.fields {
                    writeln!(f, "{}{}: {},", "  ".repeat(depth + 1), name, value)?;
                }

                // Print child entities
                for &child_id in &entity.children {
                    self.display_node(child_id, depth + 1, f)?;
                }

                // Add closing semicolon if not root
                if entity.parent.is_some() {
                    writeln!(f, "{};", "  ".repeat(depth))?;
                }
            }
            ConfigNode::Field(field) => {
                writeln!(f, "{}{}: {},", "  ".repeat(depth), field.name, field.value)?;
            }
        }

        Ok(())
    }

    /// Display the node tree recursively using tree-like ASCII characters for a beautiful representation
    pub fn pretty_display(&self) -> String {
        let mut result = String::new();
        self.pretty_display_node(self.root_id, &mut result, &mut vec![], false);
        result
    }

    /// Helper method for pretty_display to recursively build the tree representation
    fn pretty_display_node(
        &self,
        id: NodeId,
        result: &mut String,
        prefix: &mut Vec<&'static str>,
        is_last: bool,
    ) {
        let node = match self.get_node(id) {
            Some(n) => n,
            None => {
                result.push_str(&prefix.join(""));
                result.push_str(&format!("Node ID {} not found\n", id));
                return;
            }
        };

        // Print current line with the right prefix
        result.push_str(&prefix.join(""));

        // Last item gets corner, others get T-junction
        if is_last {
            result.push_str("└── ");
        } else {
            result.push_str("├── ");
        }

        let node_borrow = node.borrow();
        match &*node_borrow {
            ConfigNode::Entity(entity) => {
                // For display purposes, we check if the name represents a quoted entity with version info
                let display_name = self.get_original_entity_name(&entity.name);

                // Actually display the name (and plural if available)
                if let Some(plural) = &entity.plural_name {
                    result.push_str(&format!("{} plural {}\n", display_name, plural));
                } else {
                    result.push_str(&format!("{}\n", display_name));
                }

                // For any future children, if this was the last item, add empty space
                // otherwise add vertical line with space
                if is_last {
                    prefix.push("    ");
                } else {
                    prefix.push("│   ");
                }

                // Process fields first
                let fields: Vec<(&String, &ConfigValue)> = entity.fields.iter().collect();
                let num_fields = fields.len();

                for (i, (name, value)) in fields.into_iter().enumerate() {
                    // Determine if this is the last field
                    let is_field_last = i == num_fields - 1 && entity.children.is_empty();

                    // Print prefix
                    result.push_str(&prefix.join(""));

                    // Print field with appropriate connector
                    if is_field_last {
                        result.push_str(&format!("└── {}: {}\n", name, value));
                    } else {
                        result.push_str(&format!("├── {}: {}\n", name, value));
                    }
                }

                // Process child entities
                let num_children = entity.children.len();
                for (i, &child_id) in entity.children.iter().enumerate() {
                    let is_child_last = i == num_children - 1;
                    self.pretty_display_node(child_id, result, prefix, is_child_last);
                }

                // Remove the last prefix when we're done with this node
                prefix.pop();
            }
            ConfigNode::Field(field) => {
                result.push_str(&format!("{}: {}\n", field.name, field.value));
            }
        }
    }
}

impl fmt::Display for ConfigModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_node(self.root_id, 0, f)?;

        // Add final period for the entire config
        writeln!(f, ".")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config_model() {
        let mut model = ConfigModel::new();

        // Create 'llm plural llms' at root
        let _llm_id = model
            .create_entity_at_path("", "llm", Some("llms"), None)
            .unwrap();

        // Create 'openai' under 'llm'
        let openai_id = model
            .create_entity_at_path("llm", "openai", None, None)
            .unwrap();

        // Add fields to openai
        model
            .add_field_to_entity(
                openai_id,
                "api_key",
                ConfigValue::String("test_key".to_string()),
            )
            .unwrap();
        model
            .add_field_to_entity(openai_id, "max_tokens", ConfigValue::Integer(1000))
            .unwrap();

        // Create 'model plural models' under 'openai'
        let _models_id = model
            .create_entity_at_path("llm/openai", "model", Some("models"), None)
            .unwrap();

        // Create 'gpt-4' under 'models'
        let gpt4_id = model
            .create_entity_at_path("llm/openai/model", "gpt-4", None, None)
            .unwrap();

        // Add fields to gpt-4
        model
            .add_field_to_entity(gpt4_id, "max_input_tokens", ConfigValue::Integer(8192))
            .unwrap();
        model
            .add_field_to_entity(gpt4_id, "supports_vision", ConfigValue::Boolean(true))
            .unwrap();

        // Display the model
        println!("{}", model);

        // Test retrieving values
        let max_tokens = model.get_field_value(openai_id, "max_tokens").unwrap();
        if let ConfigValue::Integer(val) = max_tokens {
            assert_eq!(val, 1000);
        } else {
            panic!("Expected Integer value for max_tokens");
        }

        // Test finding entity by path
        let found_gpt4_id = model.find_entity_by_path("llm/openai/model/gpt-4").unwrap();
        assert_eq!(found_gpt4_id, gpt4_id);
    }
}
