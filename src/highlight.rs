// SPDX-License-Identifier: Apache-2.0
//! cola's token vocabulary, for editors.
//!
//! An error-tolerant, **line-state** lexer over the grammar's terminals
//! (`src/grammar/cola.rustemo`). cola is literate: prose, headings and
//! fenced blocks interleave, and a line lexes differently depending on
//! the fence it sits in — so the API threads a [`LineState`] from line
//! to line instead of parsing the document (a full LR parse fails on
//! the mid-edit buffers editors hold; lexing never does). Ranges are
//! byte ranges within the line. The parser stays the source of truth
//! for *meaning*; this module only names what each span *is*.
//!
//! Fence shapes come from [`crate::literate`], the same functions the
//! parser's scanner uses, so the editor and the parser agree on where a
//! cola block starts and ends.

use crate::literate::{closes, fence_opener};
use std::ops::Range;

pub use crate::literate::Fence;

/// Which region the *next* line starts in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineState {
    /// Markdown prose between fences.
    #[default]
    Prose,
    /// Inside a cola fence — entity syntax — opened with this fence.
    Cola(Fence),
    /// Inside a regular fenced code block opened with this fence.
    Code(Fence),
}

impl LineState {
    /// A bare `.cola` buffer starts here: entity syntax with no fence
    /// to close it. Three backticks are a closer that cannot appear
    /// in cola, so the state is stable.
    pub const BARE_COLA: LineState = LineState::Cola(Fence { ch: b'`', len: 3 });

    pub fn is_cola(self) -> bool {
        matches!(self, LineState::Cola(_))
    }
}

/// What a span is, in cola's own vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// A markdown heading line (`#` … `######`).
    Heading,
    /// A fence delimiter line (```` ```cola ````, ```` ``` ````, `~~~`, named opens).
    Fence,
    /// A line inside a regular (non-cola) code block.
    CodeLine,
    /// `plural` — the structural keyword.
    Keyword,
    /// `true` / `false`.
    Boolean,
    /// An identifier (entity or field name).
    Identifier,
    /// A number literal.
    Number,
    /// A quoted string (single or double).
    String,
    /// `:` `,` `;`.
    Punctuation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub range: Range<usize>,
    pub kind: TokenKind,
}

/// Tokenize one line given the state the previous line left behind;
/// returns the tokens and the state for the next line. Unrecognised
/// bytes are skipped, never an error.
pub fn line_tokens(line: &str, state: LineState) -> (Vec<Token>, LineState) {
    match state {
        LineState::Prose => {
            if let Some((fence, info)) = fence_opener(line) {
                let next = if info.split_whitespace().next() == Some("cola") {
                    LineState::Cola(fence)
                } else {
                    LineState::Code(fence)
                };
                (vec![whole(line, TokenKind::Fence)], next)
            } else if is_heading(line) {
                (vec![whole(line, TokenKind::Heading)], LineState::Prose)
            } else {
                // Prose stays unstyled — the document's own voice.
                (Vec::new(), LineState::Prose)
            }
        }
        LineState::Code(fence) => {
            if closes(line, fence) {
                (vec![whole(line, TokenKind::Fence)], LineState::Prose)
            } else {
                (vec![whole(line, TokenKind::CodeLine)], state)
            }
        }
        LineState::Cola(fence) => {
            if closes(line, fence) {
                (vec![whole(line, TokenKind::Fence)], LineState::Prose)
            } else {
                (cola_tokens(line), state)
            }
        }
    }
}

/// An ATX heading: up to three spaces, one to six `#`, then a blank or
/// the end of the line. `#tag` and `#1` are prose.
fn is_heading(line: &str) -> bool {
    let rest = line.trim_end_matches(['\n', '\r']).trim_start_matches(' ');
    if line.len() - rest.len() > 3 {
        return false;
    }
    let hashes = rest.bytes().take_while(|&b| b == b'#').count();
    (1..=6).contains(&hashes)
        && rest[hashes..]
            .bytes()
            .next()
            .is_none_or(|b| b == b' ' || b == b'\t')
}

fn whole(line: &str, kind: TokenKind) -> Token {
    Token {
        range: 0..line.trim_end_matches(['\n', '\r']).len(),
        kind,
    }
}

/// Inside a cola fence: entities, fields, values.
fn cola_tokens(line: &str) -> Vec<Token> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'"' || c == b'\'' {
            // Quoted string with escapes; unterminated runs to EOL
            // (mid-edit tolerance).
            let quote = c;
            let start = i;
            i += 1;
            while i < b.len() && b[i] != quote {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(b.len());
            out.push(Token {
                range: start..i,
                kind: TokenKind::String,
            });
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < b.len()
                && (b[i].is_ascii_alphanumeric() || matches!(b[i], b'_' | b'.' | b'-'))
            {
                i += 1;
            }
            let word = &line[start..i];
            let kind = match word {
                "plural" => TokenKind::Keyword,
                "true" | "false" => TokenKind::Boolean,
                _ => TokenKind::Identifier,
            };
            out.push(Token {
                range: start..i,
                kind,
            });
            continue;
        }
        if c.is_ascii_digit()
            || ((c == b'+' || c == b'-') && b.get(i + 1).is_some_and(u8::is_ascii_digit))
        {
            let start = i;
            i += 1;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            out.push(Token {
                range: start..i,
                kind: TokenKind::Number,
            });
            continue;
        }
        if matches!(c, b':' | b',' | b';') {
            out.push(Token {
                range: i..i + 1,
                kind: TokenKind::Punctuation,
            });
            i += 1;
            continue;
        }
        i += line[i..].chars().next().map(char::len_utf8).unwrap_or(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(doc: &str) -> Vec<(String, TokenKind)> {
        let mut state = LineState::default();
        let mut out = Vec::new();
        for line in doc.split_inclusive('\n') {
            let (toks, next) = line_tokens(line, state);
            state = next;
            for t in toks {
                out.push((line[t.range.clone()].to_string(), t.kind));
            }
        }
        out
    }

    #[test]
    fn literate_structure_and_cola_entities_lex_by_region() {
        use TokenKind::*;
        let doc = "# Domain\n\nProse explaining things.\n\n```cola\nusers plural user:\n  name: \"Alice\", age: 42, active: true;\n```\n\n```rust\nfn x() {}\n```\n";
        let got = run(doc);
        assert_eq!(got[0], ("# Domain".into(), Heading));
        // Prose produced nothing.
        assert!(!got.iter().any(|(t, _)| t.contains("Prose")));
        assert_eq!(got[1], ("```cola".into(), Fence));
        assert_eq!(got[2], ("users".into(), Identifier));
        assert_eq!(got[3], ("plural".into(), Keyword));
        assert!(got.contains(&("\"Alice\"".into(), String)));
        assert!(got.contains(&("42".into(), Number)));
        assert!(got.contains(&("true".into(), Boolean)));
        // The rust fence's body is CodeLine, not cola tokens.
        assert!(got.contains(&("fn x() {}".into(), CodeLine)));
    }

    #[test]
    fn mid_edit_unterminated_string_reaches_eol_and_recovers() {
        let (toks, next) = line_tokens("name: \"unterm\n", LineState::BARE_COLA);
        assert_eq!(toks.last().unwrap().kind, TokenKind::String);
        assert!(next.is_cola());
    }

    #[test]
    fn fence_shapes_follow_the_scanner() {
        use TokenKind::*;
        // Tilde fences, info strings, and a long fence quoting a short one.
        let doc = "~~~cola title=\"x\"\na: b: 1;\n~~~\n````md\n```cola\n````\n";
        let got = run(doc);
        assert_eq!(got[0], ("~~~cola title=\"x\"".into(), Fence));
        assert_eq!(got[1], ("a".into(), Identifier));
        assert!(got.contains(&("~~~".into(), Fence)));
        assert!(got.contains(&("```cola".into(), CodeLine)));
        // A short closer does not close a long fence.
        let long = crate::literate::Fence { ch: b'`', len: 4 };
        let (_, s) = line_tokens("```\n", LineState::Code(long));
        assert_eq!(s, LineState::Code(long));
    }

    #[test]
    fn headings_need_a_blank_after_the_hashes() {
        assert!(run("# H\n").iter().any(|(_, k)| *k == TokenKind::Heading));
        assert!(
            run("###### H\n")
                .iter()
                .any(|(_, k)| *k == TokenKind::Heading)
        );
        assert!(run("#\n").iter().any(|(_, k)| *k == TokenKind::Heading));
        assert!(run("#tag\n").is_empty());
        assert!(run("#1\n").is_empty());
        assert!(run("####### seven\n").is_empty());
    }

    /// Drift alarm: the literal terminals the grammar declares lex to
    /// the kinds this module names.
    #[test]
    fn lexer_covers_the_grammars_literal_terminals() {
        let grammar = include_str!("grammar/cola.rustemo");
        assert!(grammar.contains("PluralKeyword: \"plural\""));
        let (t, _) = line_tokens("plural", LineState::BARE_COLA);
        assert_eq!(t[0].kind, TokenKind::Keyword);
        for (lit, kind) in [("true", TokenKind::Boolean), ("false", TokenKind::Boolean)] {
            assert!(grammar.contains(&format!("'{lit}'")));
            let (t, _) = line_tokens(lit, LineState::BARE_COLA);
            assert_eq!(t[0].kind, kind, "{lit}");
        }
        for p in [":", ",", ";"] {
            assert!(grammar.contains(&format!("'{p}'")));
            let (t, _) = line_tokens(p, LineState::BARE_COLA);
            assert_eq!(t[0].kind, TokenKind::Punctuation, "{p}");
        }
    }
}
