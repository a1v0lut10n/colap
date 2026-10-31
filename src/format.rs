// SPDX-License-Identifier: Apache-2.0
//! `colap fmt`: rewrite the cola in a file into the canonical layout
//! and leave everything else alone.
//!
//! A literate file is formatted block by block — each fence body is
//! parsed on its own and written back canonically — so prose, headings
//! and foreign fences are preserved byte for byte, and a collection
//! split over several blocks stays split. The whole file is checked
//! first: a file with errors is not formatted.

use crate::error::Error;
use crate::model::model_builder::ModelBuilder;
use crate::{literate, parser::cola};
use rustemo::Parser as _;
use std::path::Path;

/// The result of formatting: the new text, and the 1-based line of
/// every block that changed (the fence line; `1` for a bare file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatted {
    pub text: String,
    pub changed: Vec<u32>,
}

impl Formatted {
    pub fn is_canonical(&self) -> bool {
        self.changed.is_empty()
    }
}

/// One block body, parsed alone and written canonically. References
/// are not resolved here: they may point into another block, and the
/// whole document was checked before any block is formatted.
fn canonical_body(body: &str, path: Option<&Path>, line_offset: u32) -> Result<String, Error> {
    let ast = cola::ColaParser::new()
        .parse(body)
        .map_err(|e| Error::from_rustemo(e, path, line_offset))?;
    let mut builder = ModelBuilder::new(path);
    builder.add_block(&ast, body, line_offset);
    Ok(builder.finish_block()?.to_cola())
}

/// Format a literate document: only the bodies of cola fences change.
pub fn format_literate_str(text: &str, path: Option<&Path>) -> Result<Formatted, Error> {
    crate::check_literate_str(text, path).outcome?;

    let scan = literate::scan(text);
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut out = String::with_capacity(text.len());
    let mut changed = Vec::new();
    let mut cursor = 0;

    for block in scan.cola_blocks() {
        // The body starts on the line after the fence: 0-based index
        // `open_line`.
        let body_start = block.open_line as usize;
        let body_lines = block.body.split_inclusive('\n').count();
        for line in &lines[cursor..body_start] {
            out.push_str(line);
        }
        let canonical = canonical_body(&block.body, path, block.body_line() - 1)?;
        if canonical != block.body {
            changed.push(block.open_line);
        }
        out.push_str(&canonical);
        cursor = body_start + body_lines;
    }
    for line in &lines[cursor..] {
        out.push_str(line);
    }

    Ok(Formatted { text: out, changed })
}

/// Format bare cola: the whole text is one block.
pub fn format_cola_str(text: &str, path: Option<&Path>) -> Result<Formatted, Error> {
    let model = crate::check_cola_str(text, path).outcome?;
    let canonical = model.to_cola();
    let changed = if canonical == text {
        Vec::new()
    } else {
        vec![1]
    };
    Ok(Formatted {
        text: canonical,
        changed,
    })
}

/// Format text whose dialect is detected as in [`crate::check_str`].
pub fn format_str(text: &str, path: Option<&Path>) -> Result<Formatted, Error> {
    if literate::scan(text).cola_blocks().next().is_some() {
        format_literate_str(text, path)
    } else {
        format_cola_str(text, path)
    }
}

/// Format a file's contents, without writing: the dialect follows the
/// extension as in [`crate::check_file`].
pub fn format_file(path: impl AsRef<Path>) -> Result<Formatted, Error> {
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
        format_literate_str(&text, Some(path))
    } else {
        format_str(&text, Some(path))
    }
}
