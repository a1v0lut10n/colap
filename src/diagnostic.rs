// SPDX-License-Identifier: Apache-2.0
//! Findings about a file: warnings for the pitfalls that would otherwise
//! be silent, and the one rendering of errors and warnings that
//! `colap check` prints.
//!
//! A [`Diagnostic`] is the flat shape downstream linters and editors
//! consume: severity, path, 1-based line and column, message. Its
//! `Display` is `path:line:col: severity: message`, with the parts that
//! are unknown left out. [`to_cola`] renders a list of findings as a
//! cola block, so a report is itself a literate document.

use crate::error::Error;
use crate::literate;
use crate::model::source_location::SourceLocation;
use std::fmt;
use std::path::{Path, PathBuf};

/// Something that is not an error but that the author probably did not
/// mean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// The scanner's own: an unterminated fence.
    Fence(literate::Warning),
    /// A literate file with no cola fence at all: nothing was read.
    NoColaBlocks,
    /// An entity with neither fields nor children.
    EmptyEntity {
        /// The entity's path from the root.
        entity: String,
        location: Option<SourceLocation>,
    },
}

impl Warning {
    /// The 1-based line and column the warning points at, when it has one.
    pub fn location(&self) -> Option<(u32, u32)> {
        match self {
            Warning::Fence(literate::Warning::UnterminatedFence { line, .. }) => Some((*line, 1)),
            Warning::NoColaBlocks => None,
            Warning::EmptyEntity { location, .. } => location
                .as_ref()
                .map(|loc| (loc.start_line, loc.start_column + 1)),
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::Fence(literate::Warning::UnterminatedFence { lang, .. }) => {
                if lang.is_empty() {
                    write!(f, "unclosed fence runs to the end of the file")
                } else {
                    write!(f, "unclosed {lang} fence runs to the end of the file")
                }
            }
            Warning::NoColaBlocks => write!(f, "no cola blocks in this file"),
            Warning::EmptyEntity { entity, .. } => write!(f, "empty entity `{entity}`"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// One finding, in the shape tools consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub path: Option<PathBuf>,
    /// 1-based.
    pub line: Option<u32>,
    /// 1-based.
    pub column: Option<u32>,
    pub message: String,
}

impl Diagnostic {
    /// A finding at a location in `path`.
    pub fn at(
        severity: Severity,
        path: Option<&Path>,
        location: Option<&SourceLocation>,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            severity,
            path: path.map(Path::to_path_buf),
            line: location.map(|l| l.start_line),
            column: location.map(|l| l.start_column + 1),
            message,
        }
    }

    /// Every error inside `error`, flattening [`Error::Multiple`].
    pub fn from_error(error: &Error) -> Vec<Diagnostic> {
        error
            .each()
            .map(|e| {
                let (line, column) = match e.location() {
                    Some((line, column)) => (Some(line), Some(column)),
                    None => (None, None),
                };
                Diagnostic {
                    severity: Severity::Error,
                    path: e.path().map(Path::to_path_buf),
                    line,
                    column,
                    message: e.message(),
                }
            })
            .collect()
    }

    pub fn from_warning(path: Option<&Path>, warning: &Warning) -> Diagnostic {
        let (line, column) = match warning.location() {
            Some((line, column)) => (Some(line), Some(column)),
            None => (None, None),
        };
        Diagnostic {
            severity: Severity::Warning,
            path: path.map(Path::to_path_buf),
            line,
            column,
            message: warning.to_string(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = String::new();
        if let Some(path) = &self.path {
            prefix.push_str(&path.display().to_string());
        }
        if let (Some(line), Some(column)) = (self.line, self.column) {
            if !prefix.is_empty() {
                prefix.push(':');
            }
            prefix.push_str(&format!("{line}:{column}"));
        }
        if prefix.is_empty() {
            write!(f, "{}: {}", self.severity, self.message)
        } else {
            write!(f, "{prefix}: {}: {}", self.severity, self.message)
        }
    }
}

fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The findings as a cola block: `finding plural findings`, one entry
/// per finding with `severity`, `path`, `line`, `column` and `message`
/// fields (the unknown ones omitted).
pub fn to_cola(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::from("```cola\nfinding plural findings:\n");
    for (i, d) in diagnostics.iter().enumerate() {
        out.push_str(&format!("    f{}:\n", i + 1));
        let mut fields = vec![format!("severity: {}", quote(&d.severity.to_string()))];
        if let Some(path) = &d.path {
            fields.push(format!("path: {}", quote(&path.display().to_string())));
        }
        if let Some(line) = d.line {
            fields.push(format!("line: {line}"));
        }
        if let Some(column) = d.column {
            fields.push(format!("column: {column}"));
        }
        fields.push(format!("message: {}", quote(&d.message)));
        let last = fields.len() - 1;
        for (j, field) in fields.iter().enumerate() {
            out.push_str("        ");
            out.push_str(field);
            out.push_str(if j < last { ",\n" } else { "\n" });
        }
        out.push_str("    ;\n");
    }
    out.push_str(";\n```\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(path: Option<&str>, line: Option<u32>, severity: Severity, message: &str) -> Diagnostic {
        Diagnostic {
            severity,
            path: path.map(PathBuf::from),
            line,
            column: line.map(|_| 5),
            message: message.into(),
        }
    }

    #[test]
    fn display_leaves_out_what_is_unknown() {
        use Severity::*;
        assert_eq!(
            d(Some("a.md"), Some(6), Error, "m").to_string(),
            "a.md:6:5: error: m"
        );
        assert_eq!(
            d(Some("a.md"), None, Warning, "m").to_string(),
            "a.md: warning: m"
        );
        assert_eq!(d(None, Some(6), Error, "m").to_string(), "6:5: error: m");
        assert_eq!(d(None, None, Warning, "m").to_string(), "warning: m");
    }

    #[test]
    fn cola_output_quotes_strings() {
        let out = to_cola(&[d(
            Some("a.md"),
            Some(6),
            Severity::Error,
            "say \"hi\" \\ bye",
        )]);
        assert!(
            out.contains("message: \"say \\\"hi\\\" \\\\ bye\""),
            "{out}"
        );
        assert!(out.starts_with("```cola\nfinding plural findings:\n    f1:\n"));
        assert!(out.ends_with("    ;\n;\n```\n"));
    }
}
