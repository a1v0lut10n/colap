// SPDX-License-Identifier: Apache-2.0
//! colap — parser and code generator for cola, a configuration language
//! that lives in markdown.
//!
//! The entry points are the `check_*` and `parse_*` functions below. A
//! literate file is markdown whose ```` ```cola ```` fences carry
//! configuration ([`literate`] finds them); a bare file is cola from the
//! first byte. Reading gives a [`ConfigModel`] whose locations point
//! into the original file.
//!
//! `check_*` never fail: they return a [`Report`] with the outcome (the
//! model, or every error in the file — see [`Error::Multiple`]) *and*
//! the warnings, which matter most when there are errors (an unclosed
//! fence is usually why the block after it did not parse). `parse_*`
//! are the convenience form that gives the model and warnings on
//! success and the error otherwise.
//!
//! The rustemo-generated parser stays reachable under [`parser`] for
//! callers that want the raw AST of one cola block, and [`rustemo`] is
//! re-exported so they need not depend on it themselves.

pub mod diagnostic;
pub mod error;
pub mod format;
pub mod generator;
pub mod grammar;
pub mod highlight;
pub mod literate;
pub mod model;
pub mod parser;

// Re-export key components for backward compatibility
pub use diagnostic::{Diagnostic, Warning};
pub use error::Error;
pub use model::config_model;
pub use model::config_model::ConfigModel;
pub use model::model_builder;
pub use model::model_builder::ModelBuilder;
pub use model::source_location;
pub use parser::cola;
pub use parser::cola_actions;
pub use rustemo;

use rustemo::Parser as _;
use std::path::{Path, PathBuf};

/// Everything one reading of a file found.
#[derive(Debug)]
pub struct Report {
    pub path: Option<PathBuf>,
    /// The model, or every error in the file.
    pub outcome: Result<ConfigModel, Error>,
    /// Independent of the outcome.
    pub warnings: Vec<Warning>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.outcome.is_ok()
    }

    /// Errors and warnings together, in line order.
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut out = match &self.outcome {
            Ok(_) => Vec::new(),
            Err(error) => Diagnostic::from_error(error),
        };
        out.extend(
            self.warnings
                .iter()
                .map(|w| Diagnostic::from_warning(self.path.as_deref(), w)),
        );
        out.sort_by_key(|d| d.line.unwrap_or(0));
        out
    }

    /// The model with its warnings, or the error.
    pub fn into_parsed(self) -> Result<Parsed, Error> {
        self.outcome.map(|model| Parsed {
            model,
            warnings: self.warnings,
        })
    }
}

/// A parsed file: its model and whatever was noticed along the way.
#[derive(Debug)]
pub struct Parsed {
    pub model: ConfigModel,
    pub warnings: Vec<Warning>,
}

/// Entities with neither fields nor children, the root excepted.
fn empty_entities(model: &ConfigModel) -> Vec<Warning> {
    model
        .node_ids()
        .filter(|&id| id != model.root_id())
        .filter_map(|id| {
            let node = model.get_node(id)?;
            let node = node.borrow();
            (node.fields.is_empty() && node.children.is_empty()).then(|| Warning::EmptyEntity {
                entity: model.entity_path(id),
                location: node.location.clone(),
            })
        })
        .collect()
}

fn finish(builder: ModelBuilder, path: Option<&Path>, mut warnings: Vec<Warning>) -> Report {
    let outcome = builder.finish();
    if let Ok(model) = &outcome {
        warnings.extend(empty_entities(model));
    }
    Report {
        path: path.map(Path::to_path_buf),
        outcome,
        warnings,
    }
}

/// Read a literate (markdown) document: only the bodies of ```` ```cola ````
/// fences are cola; everything else is prose and is never looked at.
/// `path` is recorded on locations and errors.
pub fn check_literate_str(text: &str, path: Option<&Path>) -> Report {
    let scan = literate::scan(text);
    let mut warnings: Vec<Warning> = scan.warnings.iter().cloned().map(Warning::Fence).collect();
    if scan.cola_blocks().next().is_none() {
        warnings.push(Warning::NoColaBlocks);
    }
    let mut builder = ModelBuilder::new(path);
    for block in scan.cola_blocks() {
        let line_offset = block.body_line() - 1;
        // A block that does not parse is recorded and the next one is
        // still read, so one bad block does not hide the rest.
        match cola::ColaParser::new().parse(&block.body) {
            Ok(ast) => builder.add_block(&ast, line_offset),
            Err(e) => builder.record(Error::from_rustemo(e, path, line_offset)),
        }
    }
    finish(builder, path, warnings)
}

/// Read bare cola: the whole text is entity syntax, no fences.
pub fn check_cola_str(text: &str, path: Option<&Path>) -> Report {
    let mut builder = ModelBuilder::new(path);
    match cola::ColaParser::new().parse(text) {
        Ok(ast) => builder.add_block(&ast, 0),
        Err(e) => builder.record(Error::from_rustemo(e, path, 0)),
    }
    finish(builder, path, Vec::new())
}

/// Read text whose dialect is not known from its name: literate when
/// it contains at least one cola fence, bare cola otherwise. A markdown
/// file with no cola fence therefore fails to parse instead of quietly
/// yielding an empty model; use [`check_literate_str`] when markdown is
/// certain.
pub fn check_str(text: &str, path: Option<&Path>) -> Report {
    if literate::scan(text).cola_blocks().next().is_some() {
        check_literate_str(text, path)
    } else {
        check_cola_str(text, path)
    }
}

/// Read a file. `.md` and `.markdown` files are literate; any other
/// file goes through [`check_str`]'s detection, so a `.cola` file may be
/// bare or literate.
pub fn check_file(path: impl AsRef<Path>) -> Report {
    let path = path.as_ref();
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(source) => {
            return Report {
                path: Some(path.to_path_buf()),
                outcome: Err(Error::Io {
                    path: path.to_path_buf(),
                    source,
                }),
                warnings: Vec::new(),
            };
        }
    };
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if ext == "md" || ext == "markdown" {
        check_literate_str(&text, Some(path))
    } else {
        check_str(&text, Some(path))
    }
}

/// [`check_literate_str`], as a result.
pub fn parse_literate_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    check_literate_str(text, path).into_parsed()
}

/// [`check_cola_str`], as a result.
pub fn parse_cola_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    check_cola_str(text, path).into_parsed()
}

/// [`check_str`], as a result.
pub fn parse_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    check_str(text, path).into_parsed()
}

/// [`check_file`], as a result.
pub fn parse_file(path: impl AsRef<Path>) -> Result<Parsed, Error> {
    check_file(path).into_parsed()
}
