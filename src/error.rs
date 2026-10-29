// SPDX-License-Identifier: Apache-2.0
//! colap's error type.
//!
//! Every error names the file it came from when one is known, and a
//! located error carries the line and column *in that file* — not in
//! the extracted fence body the grammar actually saw. `Display` is the
//! editor-friendly `path:line:col: message` form, with 1-based lines
//! and columns. Several errors from one parse come back together as
//! [`Error::Multiple`], sorted by line.

use crate::model::config_model::ModelError;
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
    /// The input parsed but does not make a consistent model: a
    /// duplicate, or a value the model cannot hold.
    Model {
        path: Option<PathBuf>,
        error: Box<ModelError>,
    },
    /// The file could not be read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// Several errors, in line order.
    Multiple(Vec<Error>),
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

    pub(crate) fn model(error: ModelError, path: Option<&Path>) -> Self {
        Error::Model {
            path: path.map(Path::to_path_buf),
            error: Box::new(error),
        }
    }

    /// The file this error refers to, when known.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Error::Parse { path, .. } | Error::Model { path, .. } => path.as_deref(),
            Error::Io { path, .. } => Some(path),
            Error::Multiple(errors) => errors.first().and_then(Error::path),
        }
    }

    /// The 1-based line the error is reported at, when it has one.
    pub fn line(&self) -> Option<u32> {
        match self {
            Error::Parse { line, .. } => Some(*line),
            Error::Model { error, .. } => error.location().map(|loc| loc.start_line),
            Error::Io { .. } => None,
            Error::Multiple(errors) => errors.first().and_then(Error::line),
        }
    }

    /// The 1-based line and column the error is reported at, when it
    /// has them.
    pub fn location(&self) -> Option<(u32, u32)> {
        match self {
            Error::Parse { line, column, .. } => Some((*line, *column)),
            Error::Model { error, .. } => error
                .location()
                .map(|loc| (loc.start_line, loc.start_column + 1)),
            Error::Io { .. } => None,
            Error::Multiple(errors) => errors.first().and_then(Error::location),
        }
    }

    /// The message without path or location.
    pub fn message(&self) -> String {
        match self {
            Error::Parse { message, .. } => message.clone(),
            Error::Model { error, .. } => error.to_string(),
            Error::Io { source, .. } => source.to_string(),
            Error::Multiple(errors) => errors
                .iter()
                .map(Error::message)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// Every individual error, flattening [`Error::Multiple`].
    pub fn each(&self) -> Box<dyn Iterator<Item = &Error> + '_> {
        match self {
            Error::Multiple(errors) => Box::new(errors.iter().flat_map(Error::each)),
            single => Box::new(std::iter::once(single)),
        }
    }

    /// Fold collected errors into one: none is `Ok`, one is itself,
    /// more are [`Error::Multiple`] sorted by line.
    pub(crate) fn collect(mut errors: Vec<Error>) -> Result<(), Error> {
        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.remove(0)),
            _ => {
                errors.sort_by_key(|e| e.line().unwrap_or(0));
                Err(Error::Multiple(errors))
            }
        }
    }
}

fn write_path(f: &mut fmt::Formatter<'_>, path: &Option<PathBuf>, sep: &str) -> fmt::Result {
    if let Some(path) = path {
        write!(f, "{}{sep}", path.display())?;
    }
    Ok(())
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
                write_path(f, path, ":")?;
                write!(f, "{line}:{column}: {message}")
            }
            Error::Model { path, error } => match error.location() {
                Some(loc) => {
                    write_path(f, path, ":")?;
                    write!(f, "{}:{}: {error}", loc.start_line, loc.start_column + 1)
                }
                None => {
                    write_path(f, path, ": ")?;
                    write!(f, "{error}")
                }
            },
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Multiple(errors) => {
                for (i, error) in errors.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            Error::Model { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}
