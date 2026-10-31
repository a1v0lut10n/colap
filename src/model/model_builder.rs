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
//!
//! Comments are layout to the grammar, so the parser never hands them
//! over. `add_block` takes the block text as well and finds them with
//! the highlighter's lexer, then attaches each to the model by
//! position in one walk over the AST: a comment on the line where a
//! field, an entity header or an entity's `;` ends is that item's
//! trailing comment; any other comment leads the next item of the
//! innermost entity around it, or that entity's close when no item
//! follows, or the end of the block at the top level.

use crate::error::Error;
use crate::highlight::{TokenKind, cola_tokens};
use crate::model::config_model::{Comments, ConfigModel, ConfigValue, ModelError, NodeId};
use crate::model::literal;
use crate::model::source_location::SourceLocation;
use crate::parser::cola_actions::{Cola, Entity, Field, FieldValue, Item, List, Scalar};
use std::collections::VecDeque;
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

/// A 1-based line and 0-based byte column within a block body.
type Pos = (u32, usize);

/// A comment found in a block body.
#[derive(Debug)]
struct Comment {
    pos: Pos,
    /// The text after `//`, trailing whitespace trimmed.
    text: String,
}

/// The item a comment may attach to.
#[derive(Clone, Copy)]
enum Target {
    /// An entity's header.
    Header(NodeId),
    /// A field, by index in its entity.
    Field(NodeId, usize),
    /// An entity's `;`.
    Close(NodeId),
}

/// The comments of a block not yet attached, in source order.
struct Pending(VecDeque<Comment>);

impl Pending {
    fn scan(body: &str) -> Self {
        let mut comments = VecDeque::new();
        let mut in_string = false;
        for (index, line) in body.split_inclusive('\n').enumerate() {
            let (tokens, open) = cola_tokens(line, in_string);
            in_string = open;
            comments.extend(
                tokens
                    .into_iter()
                    .filter(|t| t.kind == TokenKind::Comment)
                    .map(|t| Comment {
                        pos: (index as u32 + 1, t.range.start),
                        text: line[t.range.start + 2..t.range.end].trim_end().to_string(),
                    }),
            );
        }
        Self(comments)
    }

    /// The comments before `pos`, removed.
    fn before(&mut self, pos: Pos) -> Vec<Comment> {
        let mut out = Vec::new();
        while self.0.front().is_some_and(|c| c.pos < pos) {
            if let Some(c) = self.0.pop_front() {
                out.push(c);
            }
        }
        out
    }

    fn rest(&mut self) -> Vec<Comment> {
        self.0.drain(..).collect()
    }
}

/// The span of an AST node within the block body.
fn span(loc: Option<&rustemo::Location>) -> (Pos, Pos) {
    let Some(loc) = loc else {
        return ((0, 0), (0, 0));
    };
    let start = (loc.start.line() as u32, loc.start.column());
    let end = loc
        .end
        .map(|e| (e.line() as u32, e.column()))
        .unwrap_or(start);
    (start, end)
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

    /// Add the entities of one parsed block. `body` is the text the
    /// AST was parsed from, read again for its comments. `line_offset`
    /// is the number of file lines above the block body's first line,
    /// so a body line `n` is file line `n + line_offset`. Problems are
    /// recorded, not returned; see [`ModelBuilder::finish`].
    pub fn add_block(&mut self, cola: &Cola, body: &str, line_offset: u32) {
        self.line_offset = line_offset;
        let root_id = self.model.root_id();
        let mut pending = Pending::scan(body);
        // What a trailing comment would attach to: the last item that
        // ended, and where it ended.
        let mut prev: Option<(Target, Pos)> = None;
        if let Some(entities) = cola {
            for entity in entities {
                let (start, _) = span(entity_location(entity));
                let leading = self.settle(pending.before(start), prev, None);
                prev = self.process_entity(root_id, entity, &mut pending, leading);
            }
        }
        let at_end = self.settle(pending.rest(), prev, None);
        self.push_leading(Target::Close(root_id), at_end);
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
    /// relative to that block and no comments. The incremental API
    /// above is the one to use for literate files; see also
    /// [`crate::parse_str`].
    pub fn build_config_model(cola: &Cola) -> Result<ConfigModel, Error> {
        let mut builder = Self::new(None);
        builder.add_block(cola, "", 0);
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

    // ---- Comments ----

    /// Sort the comments found before an item: one on the line where
    /// the previous item ended is its trailing comment, one inside the
    /// previous item (a line of a multi-line list) leads it; the rest
    /// lead the next item and are returned. With no previous item (the
    /// start of a block or of an entity's definition), `header` says
    /// where the enclosing header ended so that a comment on its line
    /// trails it.
    fn settle(
        &mut self,
        comments: Vec<Comment>,
        prev: Option<(Target, Pos)>,
        header: Option<(Target, Pos)>,
    ) -> Vec<String> {
        let mut leading = Vec::new();
        for comment in comments {
            match prev.or(header) {
                Some((target, end)) if comment.pos.0 == end.0 => {
                    self.with_comments(target, |c| c.trailing = Some(comment.text));
                }
                Some((target, end)) if comment.pos < end => {
                    self.with_comments(target, |c| c.leading.push(comment.text));
                }
                _ => leading.push(comment.text),
            }
        }
        leading
    }

    fn push_leading(&mut self, target: Target, leading: Vec<String>) {
        if !leading.is_empty() {
            self.with_comments(target, |c| c.leading.extend(leading));
        }
    }

    fn with_comments(&mut self, target: Target, f: impl FnOnce(&mut Comments)) {
        let (id, index) = match target {
            Target::Header(id) | Target::Close(id) => (id, None),
            Target::Field(id, index) => (id, Some(index)),
        };
        let Some(node) = self.model.get_node(id) else {
            return;
        };
        let mut node = node.borrow_mut();
        let comments = match (target, index) {
            (Target::Header(_), _) => Some(&mut node.comments),
            (Target::Close(_), _) => Some(&mut node.closing),
            (Target::Field(..), Some(index)) => node.fields.get_mut(index).map(|f| &mut f.comments),
            (Target::Field(..), None) => None,
        };
        if let Some(comments) = comments {
            f(comments);
        }
    }

    // ---- Entities and fields ----

    /// Process an entity and add it to the model, with `leading` as
    /// its header's leading comments. Returns the entity's close and
    /// where it ended, for the comments that follow it; `None` when
    /// the entity was rejected, in which case the comments inside it
    /// are dropped with it.
    fn process_entity(
        &mut self,
        parent_id: NodeId,
        entity: &Entity,
        pending: &mut Pending,
        leading: Vec<String>,
    ) -> Option<(Target, Pos)> {
        let (entity_name, plural_name, location, header_end, definition) = match entity {
            Entity::SingularEntity(singular) => (
                singular.identifier.as_ref().trim(),
                None,
                self.location(singular.location.as_ref()),
                span(singular.identifier.location.as_ref()).1,
                &singular.entity_definition,
            ),
            Entity::PluralEntity(plural) => (
                plural.identifier_1.as_ref().trim(),
                Some(plural.identifier_3.as_ref().trim()),
                self.location(plural.location.as_ref()),
                span(plural.identifier_3.location.as_ref()).1,
                &plural.entity_definition,
            ),
        };
        let (_, end) = span(entity_location(entity));

        let entity_id = match self
            .model
            .add_entity(parent_id, entity_name, plural_name, location)
        {
            Ok(id) => id,
            Err(error) => {
                // A rejected entity takes its subtree with it: there is
                // no node to attach the contents to.
                self.reject(error);
                pending.before(end);
                return None;
            }
        };
        self.push_leading(Target::Header(entity_id), leading);

        let header = Some((Target::Header(entity_id), header_end));
        let mut prev: Option<(Target, Pos)> = None;
        for item in definition.iter().flatten() {
            let (start, item_end) = span(item_location(item));
            let leading = self.settle(pending.before(start), prev, header);
            prev = match item {
                Item::Field1(field) | Item::Field2(field) => {
                    self.add_field_to_entity(entity_id, field).map(|index| {
                        self.push_leading(Target::Field(entity_id, index), leading);
                        (Target::Field(entity_id, index), item_end)
                    })
                }
                Item::Entity(entity) => self.process_entity(entity_id, entity, pending, leading),
            };
        }
        let before_close = self.settle(pending.before(end), prev, header);
        self.push_leading(Target::Close(entity_id), before_close);
        Some((Target::Close(entity_id), end))
    }

    /// Add a field to an entity in the model; its index when added.
    fn add_field_to_entity(&mut self, entity_id: NodeId, field: &Field) -> Option<usize> {
        let field_name = field.identifier.as_ref().trim().to_string();
        let location = self.location(field.location.as_ref());
        let field_value = match Self::convert_field_value(&field.field_value) {
            Ok(value) => value,
            Err(message) => {
                self.reject(ModelError::InvalidValue { message, location });
                return None;
            }
        };
        match self
            .model
            .add_field(entity_id, &field_name, field_value, location)
        {
            Ok(()) => self
                .model
                .get_node(entity_id)
                .map(|node| node.borrow().fields.len() - 1),
            Err(error) => {
                self.reject(error);
                None
            }
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

fn entity_location(entity: &Entity) -> Option<&rustemo::Location> {
    match entity {
        Entity::SingularEntity(s) => s.location.as_ref(),
        Entity::PluralEntity(p) => p.location.as_ref(),
    }
}

fn item_location(item: &Item) -> Option<&rustemo::Location> {
    match item {
        Item::Field1(f) | Item::Field2(f) => f.location.as_ref(),
        Item::Entity(e) => entity_location(e),
    }
}
