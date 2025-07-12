// SPDX-License-Identifier: Apache-2.0
// No external imports needed
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl SourceLocation {
    pub fn new(file_path: String, start_pos: (u32, u32), end_pos: (u32, u32)) -> Self {
        Self {
            file_path: PathBuf::from(file_path),
            start_line: start_pos.0,
            start_column: start_pos.1,
            end_line: end_pos.0,
            end_column: end_pos.1,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: [({},{})-({},{})]",
            self.file_path.display(),
            self.start_line,
            self.start_column,
            self.end_line,
            self.end_column
        )
    }
}
