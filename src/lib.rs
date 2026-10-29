// SPDX-License-Identifier: Apache-2.0
//! colap — parser and code generator for cola, a configuration language
//! that lives in markdown.
//!
//! The entry points are the `parse_*` functions below. A literate file
//! is markdown whose ```` ```cola ```` fences carry configuration
//! ([`literate`] finds them); a bare file is cola from the first byte.
//! Both give a [`ConfigModel`] whose locations point into the original
//! file, plus the scanner's [`literate::Warning`]s.
//!
//! The rustemo-generated parser stays reachable under [`parser`] for
//! callers that want the raw AST of one cola block, and [`rustemo`] is
//! re-exported so they need not depend on it themselves.

pub mod error;
pub mod generator;
pub mod grammar;
pub mod highlight;
pub mod literate;
pub mod model;
pub mod parser;

// Re-export key components for backward compatibility
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
use std::path::Path;

/// A parsed file: its model and whatever the scanner noticed.
#[derive(Debug)]
pub struct Parsed {
    pub model: ConfigModel,
    pub warnings: Vec<literate::Warning>,
}

/// Parse a literate (markdown) document: only the bodies of ```` ```cola ````
/// fences are cola; everything else is prose and is never looked at.
/// `path` is recorded on locations and errors.
pub fn parse_literate_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    let scan = literate::scan(text);
    let mut builder = ModelBuilder::new(path);
    for block in scan.cola_blocks() {
        let line_offset = block.body_line() - 1;
        let ast = cola::ColaParser::new()
            .parse(&block.body)
            .map_err(|e| Error::from_rustemo(e, path, line_offset))?;
        builder.add_block(&ast, line_offset)?;
    }
    Ok(Parsed {
        model: builder.finish(),
        warnings: scan.warnings,
    })
}

/// Parse bare cola: the whole text is entity syntax, no fences.
pub fn parse_cola_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    let ast = cola::ColaParser::new()
        .parse(text)
        .map_err(|e| Error::from_rustemo(e, path, 0))?;
    let mut builder = ModelBuilder::new(path);
    builder.add_block(&ast, 0)?;
    Ok(Parsed {
        model: builder.finish(),
        warnings: Vec::new(),
    })
}

/// Parse text whose dialect is not known from its name: literate when
/// it contains at least one cola fence, bare cola otherwise. A markdown
/// file with no cola fence therefore fails to parse instead of quietly
/// yielding an empty model; use [`parse_literate_str`] when markdown is
/// certain.
pub fn parse_str(text: &str, path: Option<&Path>) -> Result<Parsed, Error> {
    if literate::scan(text).cola_blocks().next().is_some() {
        parse_literate_str(text, path)
    } else {
        parse_cola_str(text, path)
    }
}

/// Parse a file. `.md` and `.markdown` files are literate; any other
/// file goes through [`parse_str`]'s detection, so a `.cola` file may be
/// bare or literate.
pub fn parse_file(path: impl AsRef<Path>) -> Result<Parsed, Error> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if ext == "md" || ext == "markdown" {
        parse_literate_str(&text, Some(path))
    } else {
        parse_str(&text, Some(path))
    }
}
