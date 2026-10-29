// SPDX-License-Identifier: Apache-2.0
//! The configuration model: an arena of entities.
//!
//! Every entity has one namespace shared by its fields and its child
//! entities, and the model enforces the duplicate rules decided in
//! `docs/design/2026-10-29-duplicate-key-semantics.md`:
//!
//! - a name directly under a plural is a **key** and is defined once;
//! - any other entity name is a **slot**; reopening it with the same
//!   declaration continues the same node, with any other declaration
//!   it is an error;
//! - a field is set once, a name is a field or an entity but not both,
//!   and a plural declaration reserves both of its names.
//!
//! Fields and children keep their source order.

use crate::model::source_location::SourceLocation;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub type NodeId = usize;
pub type NodeRef = Rc<RefCell<EntityNode>>;

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

/// A field of an entity: its name, value and where it was set.
#[derive(Debug, Clone)]
pub struct ConfigField {
    pub name: String,
    pub value: ConfigValue,
    pub location: Option<SourceLocation>,
}

/// What a name in an entity's namespace refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Member {
    /// Index into `fields`.
    Field(usize),
    /// A child entity, by its singular or its plural name.
    Child(NodeId),
}

/// An entity: a singular section, a plural collection, or an entry of one.
#[derive(Debug)]
pub struct EntityNode {
    /// Singular form name
    pub name: String,
    /// Plural form, when this entity is a collection
    pub plural_name: Option<String>,
    pub parent: Option<NodeId>,
    /// Child entities, in source order
    pub children: Vec<NodeId>,
    /// Fields, in source order
    pub fields: Vec<ConfigField>,
    /// Where the entity was first opened
    pub location: Option<SourceLocation>,
    /// Every name taken in this entity: fields, children by singular and
    /// by plural name. One namespace.
    names: HashMap<String, Member>,
}

impl EntityNode {
    pub fn new(
        name: &str,
        plural_name: Option<&str>,
        parent: Option<NodeId>,
        location: Option<SourceLocation>,
    ) -> Self {
        Self {
            name: name.to_string(),
            plural_name: plural_name.map(str::to_string),
            parent,
            children: Vec::new(),
            fields: Vec::new(),
            location,
            names: HashMap::new(),
        }
    }

    /// Whether this entity is a collection, so that its children are keys.
    pub fn is_plural(&self) -> bool {
        self.plural_name.is_some()
    }

    /// The entity as declared: `name` or `name plural names`.
    pub fn declaration(&self) -> String {
        declaration(&self.name, self.plural_name.as_deref())
    }

    /// A field by name.
    pub fn field(&self, name: &str) -> Option<&ConfigField> {
        match self.names.get(name)? {
            Member::Field(i) => self.fields.get(*i),
            Member::Child(_) => None,
        }
    }

    /// A field's value by name.
    pub fn field_value(&self, name: &str) -> Option<&ConfigValue> {
        self.field(name).map(|f| &f.value)
    }

    /// A child entity by its singular or plural name.
    pub fn child(&self, name: &str) -> Option<NodeId> {
        match self.names.get(name)? {
            Member::Child(id) => Some(*id),
            Member::Field(_) => None,
        }
    }
}

fn declaration(name: &str, plural_name: Option<&str>) -> String {
    match plural_name {
        Some(plural) => format!("{name} plural {plural}"),
        None => name.to_string(),
    }
}

/// Which duplicate rule a name broke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateKind {
    /// A field set twice in one entity.
    Field,
    /// An entry defined twice under one plural.
    Key,
    /// A slot reopened with a different declaration.
    Declaration,
    /// A name used as both a field and an entity, or taken by a plural name.
    Name,
}

/// A duplicate, with both places it happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duplicate {
    pub kind: DuplicateKind,
    pub message: String,
    /// Where the name was first taken.
    pub first: Option<SourceLocation>,
    /// Where it was taken again: the error's own location.
    pub second: Option<SourceLocation>,
}

impl fmt::Display for Duplicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(first) = &self.first {
            write!(
                f,
                "; first at {}:{}",
                first.start_line,
                first.start_column + 1
            )?;
        }
        Ok(())
    }
}

/// What can go wrong when building a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    NoSuchNode(NodeId),
    NoSuchPath(String),
    Duplicate(Duplicate),
    /// A literal the grammar accepted but the model cannot hold.
    InvalidValue {
        message: String,
        location: Option<SourceLocation>,
    },
}

impl ModelError {
    /// The location the error is reported at, when it has one.
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            ModelError::Duplicate(d) => d.second.as_ref(),
            ModelError::InvalidValue { location, .. } => location.as_ref(),
            ModelError::NoSuchNode(_) | ModelError::NoSuchPath(_) => None,
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::NoSuchNode(id) => write!(f, "no entity with id {id}"),
            ModelError::NoSuchPath(path) => write!(f, "no entity at `{path}`"),
            ModelError::Duplicate(d) => write!(f, "{d}"),
            ModelError::InvalidValue { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ModelError {}

fn duplicate(
    kind: DuplicateKind,
    message: String,
    first: Option<SourceLocation>,
    second: Option<SourceLocation>,
) -> ModelError {
    ModelError::Duplicate(Duplicate {
        kind,
        message,
        first,
        second,
    })
}

/// The model that holds the entire configuration structure
#[derive(Debug)]
pub struct ConfigModel {
    nodes: Vec<NodeRef>,
    root_id: NodeId,
    original_entity_names: HashMap<String, String>, // Added to store original quoted entity names
}

impl Default for ConfigModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigModel {
    /// Create a new empty model
    pub fn new() -> Self {
        let mut model = ConfigModel {
            nodes: Vec::new(),
            root_id: 0,
            original_entity_names: HashMap::new(),
        };
        model.root_id = model.add_node(EntityNode::new("root", None, None, None));
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

    fn add_node(&mut self, node: EntityNode) -> NodeId {
        self.nodes.push(Rc::new(RefCell::new(node)));
        self.nodes.len() - 1
    }

    /// Get a node by its ID
    pub fn get_node(&self, id: NodeId) -> Option<NodeRef> {
        self.nodes.get(id).cloned()
    }

    /// Get the root node ID
    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    /// The path of an entity from the root, singular names joined by `/`;
    /// empty for the root.
    pub fn entity_path(&self, id: NodeId) -> String {
        let mut names = Vec::new();
        let mut current = id;
        while let Some(node) = self.get_node(current) {
            let node = node.borrow();
            let Some(parent) = node.parent else { break };
            names.push(node.name.clone());
            current = parent;
        }
        names.reverse();
        names.join("/")
    }

    /// How an entity is named in messages.
    fn describe(&self, id: NodeId) -> String {
        let path = self.entity_path(id);
        if path.is_empty() {
            "the root".to_string()
        } else {
            format!("`{path}`")
        }
    }

    fn location_of(&self, id: NodeId) -> Option<SourceLocation> {
        self.get_node(id).and_then(|n| n.borrow().location.clone())
    }

    fn member_location(&self, parent: &EntityNode, member: Member) -> Option<SourceLocation> {
        match member {
            Member::Field(i) => parent.fields.get(i).and_then(|f| f.location.clone()),
            Member::Child(id) => self.location_of(id),
        }
    }

    /// Add an entity under `parent_id`, applying the duplicate rules.
    /// Reopening a slot with the same declaration returns the existing
    /// entity's id.
    pub fn add_entity(
        &mut self,
        parent_id: NodeId,
        name: &str,
        plural_name: Option<&str>,
        location: Option<SourceLocation>,
    ) -> Result<NodeId, ModelError> {
        let parent = self
            .get_node(parent_id)
            .ok_or(ModelError::NoSuchNode(parent_id))?;
        let existing = parent.borrow().names.get(name).copied();

        match existing {
            Some(member @ Member::Field(_)) => Err(duplicate(
                DuplicateKind::Name,
                format!(
                    "`{name}` in {} is already a field and cannot also be an entity",
                    self.describe(parent_id)
                ),
                self.member_location(&parent.borrow(), member),
                location,
            )),
            Some(Member::Child(id)) => {
                let child = self.get_node(id).ok_or(ModelError::NoSuchNode(id))?;
                let child = child.borrow();
                if parent.borrow().is_plural() {
                    Err(duplicate(
                        DuplicateKind::Key,
                        format!("duplicate entry `{name}` in {}", self.describe(parent_id)),
                        child.location.clone(),
                        location,
                    ))
                } else if child.name == name && child.plural_name.as_deref() == plural_name {
                    Ok(id)
                } else {
                    Err(duplicate(
                        DuplicateKind::Declaration,
                        format!(
                            "`{name}` in {} is declared `{}` and cannot be reopened as `{}`",
                            self.describe(parent_id),
                            child.declaration(),
                            declaration(name, plural_name)
                        ),
                        child.location.clone(),
                        location,
                    ))
                }
            }
            None => {
                if let Some(plural) = plural_name
                    && let Some(member) = parent.borrow().names.get(plural).copied()
                {
                    return Err(duplicate(
                        DuplicateKind::Name,
                        format!(
                            "`{plural}` in {} is already taken and cannot be the plural name of `{name}`",
                            self.describe(parent_id)
                        ),
                        self.member_location(&parent.borrow(), member),
                        location,
                    ));
                }
                let id = self.add_node(EntityNode::new(
                    name,
                    plural_name,
                    Some(parent_id),
                    location,
                ));
                let mut parent = parent.borrow_mut();
                parent.children.push(id);
                parent.names.insert(name.to_string(), Member::Child(id));
                if let Some(plural) = plural_name {
                    parent.names.insert(plural.to_string(), Member::Child(id));
                }
                Ok(id)
            }
        }
    }

    /// Set a field on an entity, applying the duplicate rules.
    pub fn add_field(
        &mut self,
        entity_id: NodeId,
        name: &str,
        value: ConfigValue,
        location: Option<SourceLocation>,
    ) -> Result<(), ModelError> {
        let node = self
            .get_node(entity_id)
            .ok_or(ModelError::NoSuchNode(entity_id))?;
        let existing = node.borrow().names.get(name).copied();

        match existing {
            Some(member @ Member::Field(_)) => Err(duplicate(
                DuplicateKind::Field,
                format!("duplicate field `{name}` in {}", self.describe(entity_id)),
                self.member_location(&node.borrow(), member),
                location,
            )),
            Some(member @ Member::Child(_)) => Err(duplicate(
                DuplicateKind::Name,
                format!(
                    "`{name}` in {} is already an entity and cannot also be a field",
                    self.describe(entity_id)
                ),
                self.member_location(&node.borrow(), member),
                location,
            )),
            None => {
                let mut node = node.borrow_mut();
                let index = node.fields.len();
                node.fields.push(ConfigField {
                    name: name.to_string(),
                    value,
                    location,
                });
                node.names.insert(name.to_string(), Member::Field(index));
                Ok(())
            }
        }
    }

    /// Set a field without a location.
    pub fn add_field_to_entity(
        &mut self,
        entity_id: NodeId,
        field_name: &str,
        value: ConfigValue,
    ) -> Result<(), ModelError> {
        self.add_field(entity_id, field_name, value, None)
    }

    /// Find an entity by path (e.g., "llm/openai")
    pub fn find_entity_by_path(&self, path: &str) -> Option<NodeId> {
        if path.is_empty() {
            return Some(self.root_id);
        }
        let mut current = self.root_id;
        for component in path.split('/') {
            current = self.get_node(current)?.borrow().child(component)?;
        }
        Some(current)
    }

    /// Create an entity under the entity at `path` ("" for the root).
    pub fn create_entity_at_path(
        &mut self,
        path: &str,
        name: &str,
        plural_name: Option<&str>,
        location: Option<SourceLocation>,
    ) -> Result<NodeId, ModelError> {
        let parent_id = self
            .find_entity_by_path(path)
            .ok_or_else(|| ModelError::NoSuchPath(path.to_string()))?;
        self.add_entity(parent_id, name, plural_name, location)
    }

    /// Get a field value from an entity
    pub fn get_field_value(&self, entity_id: NodeId, field_name: &str) -> Option<ConfigValue> {
        self.get_node(entity_id)?
            .borrow()
            .field_value(field_name)
            .cloned()
    }

    /// Find a child entity by name within a parent entity
    pub fn find_child_entity_by_name(&self, parent_id: NodeId, child_name: &str) -> Option<NodeId> {
        self.get_node(parent_id)?.borrow().child(child_name)
    }

    /// Display the node tree recursively
    fn display_node(&self, id: NodeId, depth: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(node) = self.get_node(id) else {
            return writeln!(f, "{}Node ID {} not found", "  ".repeat(depth), id);
        };
        let entity = node.borrow();
        let indent = "  ".repeat(depth);

        writeln!(
            f,
            "{indent}{}:",
            declaration(
                &self.get_original_entity_name(&entity.name),
                entity.plural_name.as_deref()
            )
        )?;
        for field in &entity.fields {
            writeln!(f, "{indent}  {}: {},", field.name, field.value)?;
        }
        for &child_id in &entity.children {
            self.display_node(child_id, depth + 1, f)?;
        }
        if entity.parent.is_some() {
            writeln!(f, "{indent};")?;
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
        result.push_str(&prefix.join(""));
        result.push_str(if is_last { "└── " } else { "├── " });

        let Some(node) = self.get_node(id) else {
            result.push_str(&format!("Node ID {} not found\n", id));
            return;
        };
        let entity = node.borrow();

        result.push_str(&declaration(
            &self.get_original_entity_name(&entity.name),
            entity.plural_name.as_deref(),
        ));
        result.push('\n');

        prefix.push(if is_last { "    " } else { "│   " });

        let num_fields = entity.fields.len();
        for (i, field) in entity.fields.iter().enumerate() {
            let is_field_last = i == num_fields - 1 && entity.children.is_empty();
            result.push_str(&prefix.join(""));
            result.push_str(if is_field_last {
                "└── "
            } else {
                "├── "
            });
            result.push_str(&format!("{}: {}\n", field.name, field.value));
        }

        let num_children = entity.children.len();
        for (i, &child_id) in entity.children.iter().enumerate() {
            self.pretty_display_node(child_id, result, prefix, i == num_children - 1);
        }

        prefix.pop();
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

        // Test finding entity by path, by singular and by plural name
        let found_gpt4_id = model.find_entity_by_path("llm/openai/model/gpt-4").unwrap();
        assert_eq!(found_gpt4_id, gpt4_id);
        assert_eq!(
            model.find_entity_by_path("llms/openai/models/gpt-4"),
            Some(gpt4_id)
        );
        assert_eq!(model.entity_path(gpt4_id), "llm/openai/model/gpt-4");
    }

    #[test]
    fn slots_reopen_and_keys_do_not() {
        let mut model = ConfigModel::new();
        let root = model.root_id();
        let claims = model
            .add_entity(root, "claim", Some("claims"), None)
            .unwrap();
        let again = model
            .add_entity(root, "claim", Some("claims"), None)
            .unwrap();
        assert_eq!(claims, again, "a reopened plural is the same node");

        model.add_entity(claims, "cl-1", None, None).unwrap();
        let err = model.add_entity(claims, "cl-1", None, None).unwrap_err();
        assert!(matches!(
            err,
            ModelError::Duplicate(Duplicate {
                kind: DuplicateKind::Key,
                ..
            })
        ));
        assert_eq!(err.to_string(), "duplicate entry `cl-1` in `claim`");

        let err = model
            .add_entity(root, "claim", Some("claimz"), None)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "`claim` in the root is declared `claim plural claims` and cannot be reopened as `claim plural claimz`"
        );
    }

    #[test]
    fn one_namespace_per_entity() {
        let mut model = ConfigModel::new();
        let root = model.root_id();
        let llm = model.add_entity(root, "llm", None, None).unwrap();
        model
            .add_field_to_entity(llm, "api", ConfigValue::Integer(1))
            .unwrap();
        let err = model.add_entity(llm, "api", None, None).unwrap_err();
        assert_eq!(
            err.to_string(),
            "`api` in `llm` is already a field and cannot also be an entity"
        );
        let err = model
            .add_field_to_entity(llm, "api", ConfigValue::Integer(2))
            .unwrap_err();
        assert_eq!(err.to_string(), "duplicate field `api` in `llm`");

        model
            .add_field_to_entity(llm, "models", ConfigValue::Integer(3))
            .unwrap();
        let err = model
            .add_entity(llm, "model", Some("models"), None)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "`models` in `llm` is already taken and cannot be the plural name of `model`"
        );
    }
}
