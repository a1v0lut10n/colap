// SPDX-License-Identifier: Apache-2.0
//! colap's error type.
//!
//! Every error names the file it came from when one is known, and a
//! parse error carries the line and column *in that file* — not in the
//! extracted fence body the grammar actually saw. `Display` is the
//! editor-friendly `path:line:col: message` form, with 1-based lines
//! and columns.

use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    /// The grammar rejected the input.
    Parse {
        path: Option<PathBuf>,
        /// 1-based line in the original file.
        line: u32,
        /// 1-based column in the original file.
        column: u32,
        message: String,
    },
    /// The input parsed but does not make a consistent model.
    Model {
        path: Option<PathBuf>,
        message: String,
    },
    /// The file could not be read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Error {
    /// Lift a rustemo parse error on a fence body to the file's own
    /// coordinates. `line_offset` is the number of file lines above
    /// the body's first line.
    pub(crate) fn from_rustemo(err: rustemo::Error, path: Option<&Path>, line_offset: u32) -> Self {
        match err {
            rustemo::Error::Error {
                message, location, ..
            } => {
                let (line, column) = location
                    .map(|loc| (loc.start.line() as u32, loc.start.column() as u32))
                    .unwrap_or((1, 0));
                Error::Parse {
                    path: path.map(Path::to_path_buf),
                    line: line + line_offset,
                    column: column + 1,
                    // rustemo prefixes the message with a context excerpt;
                    // the location already says where, so keep the verdict.
                    message: message
                        .lines()
                        .rev()
                        .find(|l| !l.trim().is_empty())
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                }
            }
            rustemo::Error::IOError(source) => Error::Io {
                path: path.map(Path::to_path_buf).unwrap_or_default(),
                source,
            },
        }
    }

    pub(crate) fn model(message: String, path: Option<&Path>) -> Self {
        Error::Model {
            path: path.map(Path::to_path_buf),
            message,
        }
    }

    /// The file this error refers to, when known.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Error::Parse { path, .. } | Error::Model { path, .. } => path.as_deref(),
            Error::Io { path, .. } => Some(path),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse {
                path,
                line,
                column,
                message,
            } => {
                if let Some(path) = path {
                    write!(f, "{}:", path.display())?;
                }
                write!(f, "{line}:{column}: {message}")
            }
            Error::Model { path, message } => {
                if let Some(path) = path {
                    write!(f, "{}: ", path.display())?;
                }
                write!(f, "{message}")
            }
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
