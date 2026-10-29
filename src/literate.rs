// SPDX-License-Identifier: Apache-2.0
//! Markdown as a container: the fence scanner.
//!
//! A literate cola file is markdown whose ```` ```cola ```` fences carry
//! configuration. Everything else — prose, headings, tables, foreign
//! fences — is the document's own business and never reaches the
//! grammar. This module finds the fenced blocks by CommonMark's rules
//! (§4.5, "Fenced code blocks") and hands their bodies on verbatim,
//! each with the file line it starts on, so every location the parser
//! reports can be mapped back to the original file.
//!
//! The rules, as implemented:
//!
//! - an opening fence is a run of three or more backticks or tildes,
//!   indented by at most three spaces, followed by an *info string*
//!   whose first word is the language (a backtick fence's info string
//!   may not itself contain a backtick);
//! - a closing fence uses the same character, is at least as long as
//!   the opener, is indented by at most three spaces and carries
//!   nothing but blanks after it;
//! - an unterminated fence runs to the end of the file and is reported
//!   as a [`Warning`];
//! - body lines are kept exactly as written (CommonMark would strip the
//!   opener's indentation; cola ignores whitespace, and keeping the
//!   bytes keeps columns exact).
//!
//! [`fence_opener`] and [`closes`] are the single definition of a fence
//! shape: the parser's scanner and the editor highlighter both call
//! them, so they cannot disagree.

use std::fmt;

/// The shape of an opening fence: its character and run length. A
/// closer must match the character and be at least as long.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fence {
    /// `b'`'` or `b'~'`.
    pub ch: u8,
    /// Run length, at least 3.
    pub len: usize,
}

/// One fenced code block, of any language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub fence: Fence,
    /// The info string after the opener, trimmed.
    pub info: String,
    /// The first word of the info string; empty for a bare fence.
    pub lang: String,
    /// 1-based line of the opening fence in the file.
    pub open_line: u32,
    /// The body, verbatim: every line as written, newline included.
    pub body: String,
    /// False when the file ended before a closing fence.
    pub closed: bool,
}

impl Block {
    /// Whether this block carries cola.
    pub fn is_cola(&self) -> bool {
        self.lang == "cola"
    }

    /// 1-based file line of the body's first line.
    pub fn body_line(&self) -> u32 {
        self.open_line + 1
    }
}

/// Something the scanner noticed that is not an error: the file still
/// parses, but the author probably did not mean it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Warning {
    /// A fence was opened and never closed; its block runs to the end
    /// of the file.
    UnterminatedFence { line: u32, lang: String },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::UnterminatedFence { line, lang } if lang.is_empty() => {
                write!(f, "unclosed fence opened at line {line}")
            }
            Warning::UnterminatedFence { line, lang } => {
                write!(f, "unclosed {lang} fence opened at line {line}")
            }
        }
    }
}

/// The result of scanning a file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scan {
    pub blocks: Vec<Block>,
    pub warnings: Vec<Warning>,
}

impl Scan {
    /// The blocks that carry cola, in file order.
    pub fn cola_blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter().filter(|b| b.is_cola())
    }
}

/// Up to three spaces of indentation are allowed before a fence; the
/// fourth makes it an indented code block, which is literal text.
fn after_indent(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches(' ');
    (line.len() - rest.len() <= 3).then_some(rest)
}

/// Does this line open a fence? Returns the fence and the raw info
/// string (untrimmed) that follows it.
pub fn fence_opener(line: &str) -> Option<(Fence, &str)> {
    let rest = after_indent(line.trim_end_matches(['\n', '\r']))?;
    let ch = *rest.as_bytes().first()?;
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let len = rest.bytes().take_while(|&b| b == ch).count();
    if len < 3 {
        return None;
    }
    let info = &rest[len..];
    // A backtick fence cannot carry backticks in its info string: that
    // line is an inline code span, not a fence (CommonMark example 145).
    if ch == b'`' && info.contains('`') {
        return None;
    }
    Some((Fence { ch, len }, info))
}

/// Does this line close a block opened with `fence`?
pub fn closes(line: &str, fence: Fence) -> bool {
    let Some(rest) = after_indent(line.trim_end_matches(['\n', '\r'])) else {
        return false;
    };
    let run = rest.bytes().take_while(|&b| b == fence.ch).count();
    run >= fence.len && rest[run..].bytes().all(|b| b == b' ' || b == b'\t')
}

/// Scan a markdown document for fenced code blocks.
pub fn scan(text: &str) -> Scan {
    let mut scan = Scan::default();
    let mut open: Option<Block> = None;

    for (index, line) in text.split_inclusive('\n').enumerate() {
        let line_no = index as u32 + 1;
        match open.as_mut() {
            None => {
                if let Some((fence, info)) = fence_opener(line) {
                    let info = info.trim().to_string();
                    let lang = info.split_whitespace().next().unwrap_or("").to_string();
                    open = Some(Block {
                        fence,
                        info,
                        lang,
                        open_line: line_no,
                        body: String::new(),
                        closed: false,
                    });
                }
            }
            Some(block) => {
                if closes(line, block.fence) {
                    block.closed = true;
                    scan.blocks.extend(open.take());
                } else {
                    block.body.push_str(line);
                }
            }
        }
    }

    if let Some(block) = open {
        scan.warnings.push(Warning::UnterminatedFence {
            line: block.open_line,
            lang: block.lang.clone(),
        });
        scan.blocks.push(block);
    }
    scan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn langs(text: &str) -> Vec<String> {
        scan(text).blocks.into_iter().map(|b| b.lang).collect()
    }

    #[test]
    fn backtick_and_tilde_fences_open_and_close() {
        let text = "a\n```cola\nx: y: 1;\n```\nb\n~~~text\nhello\n~~~\nc\n";
        let s = scan(text);
        assert_eq!(s.warnings, vec![]);
        assert_eq!(s.blocks.len(), 2);
        assert_eq!(s.blocks[0].lang, "cola");
        assert_eq!(s.blocks[0].open_line, 2);
        assert_eq!(s.blocks[0].body_line(), 3);
        assert_eq!(s.blocks[0].body, "x: y: 1;\n");
        assert!(s.blocks[0].closed);
        assert_eq!(s.blocks[1].lang, "text");
        assert_eq!(s.blocks[1].fence.ch, b'~');
        assert_eq!(s.blocks[1].body, "hello\n");
    }

    #[test]
    fn closer_must_be_at_least_as_long_and_the_same_character() {
        // A four-backtick fence can quote a three-backtick example.
        let text = "````md\n```cola\na: b: 1;\n```\n````\n";
        let s = scan(text);
        assert_eq!(s.blocks.len(), 1);
        assert_eq!(s.blocks[0].body, "```cola\na: b: 1;\n```\n");
        // A tilde line does not close a backtick fence.
        let text = "```cola\na: b: 1;\n~~~\n```\n";
        assert_eq!(scan(text).blocks[0].body, "a: b: 1;\n~~~\n");
    }

    #[test]
    fn info_strings_and_language_tags() {
        assert_eq!(langs("```json5\n```\n"), ["json5"]);
        assert_eq!(langs("```c++\n```\n"), ["c++"]);
        assert_eq!(langs("```objective-c\n```\n"), ["objective-c"]);
        let s = scan("```cola title=\"x\" {.class}\na: b: 1;\n```\n");
        assert_eq!(s.blocks[0].lang, "cola");
        assert_eq!(s.blocks[0].info, "cola title=\"x\" {.class}");
        assert!(s.blocks[0].is_cola());
        assert_eq!(langs("```\nbare\n```\n"), [""]);
    }

    #[test]
    fn indentation_rules() {
        assert_eq!(langs("   ```cola\n   ```\n"), ["cola"]);
        // Four spaces is an indented code block, not a fence.
        assert_eq!(langs("    ```cola\n    ```\n"), Vec::<String>::new());
        // A closer may be indented differently from its opener.
        let s = scan("```cola\na: b: 1;\n   ```\n");
        assert!(s.blocks[0].closed);
    }

    #[test]
    fn a_backtick_info_string_is_an_inline_code_span() {
        assert_eq!(langs("```foo``bar```\n"), Vec::<String>::new());
        assert_eq!(fence_opener("```x`"), None);
        assert!(fence_opener("~~~x`").is_some());
    }

    #[test]
    fn unterminated_fence_runs_to_eof_with_a_warning() {
        let s = scan("# T\n\n```cola\na: b: 1;\n");
        assert_eq!(s.blocks.len(), 1);
        assert!(!s.blocks[0].closed);
        assert_eq!(s.blocks[0].body, "a: b: 1;\n");
        assert_eq!(
            s.warnings,
            vec![Warning::UnterminatedFence {
                line: 3,
                lang: "cola".into()
            }]
        );
        assert_eq!(
            s.warnings[0].to_string(),
            "unclosed cola fence opened at line 3"
        );
    }

    #[test]
    fn closing_fence_at_eof_without_newline() {
        let s = scan("```cola\na: b: 1;\n```");
        assert!(s.blocks[0].closed);
        assert!(s.warnings.is_empty());
        let s = scan("```cola\na: b: 1;");
        assert!(!s.blocks[0].closed);
        assert_eq!(s.blocks[0].body, "a: b: 1;");
    }

    #[test]
    fn crlf_lines_are_fences_too() {
        let s = scan("```cola\r\na: b: 1;\r\n```\r\n");
        assert!(s.blocks[0].closed);
        assert_eq!(s.blocks[0].body, "a: b: 1;\r\n");
    }

    #[test]
    fn prose_never_opens_anything() {
        let text = "`code` first\n``double`` span\n#tag\n#1\n####### seven\n| a |\n> q\n<b>x</b>\n- item\n42\nÜber\n";
        assert!(scan(text).blocks.is_empty());
    }
}
