// SPDX-License-Identifier: Apache-2.0
//! The conformance corpus (`tests/conformance/{valid,invalid}/`) holds
//! the parser, the diagnostics, the highlighter and the spec to each
//! other. Each file may carry a `NAME.expected` sidecar: its diagnostics,
//! one per line, with `<path>` standing for the file's path. A valid
//! file has no errors; an invalid file has at least one.
use colap::highlight::{LineState, TokenKind, line_tokens};
use colap::literate::scan;
use colap::parser::cola::ColaParser;
use colap::parser::cola_actions::{
    Cola, Entity, EntityDefinition, Field, FieldList, FieldValue, NestedBlock,
};
use colap::rustemo::{Parser as _, ValLoc};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn corpus(dir: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "md" || e == "cola")
        })
        .collect();
    files.sort();
    assert!(!files.is_empty(), "{dir} is empty");
    files
}

/// The sidecar's lines with `<path>` filled in; none when there is no sidecar.
fn expected(path: &Path) -> Vec<String> {
    let sidecar = PathBuf::from(format!("{}.expected", path.display()));
    match std::fs::read_to_string(&sidecar) {
        Ok(text) => text
            .lines()
            .map(|l| l.replace("<path>", &path.display().to_string()))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn diagnostics(path: &Path) -> (bool, Vec<String>) {
    let report = colap::check_file(path);
    let lines = report.diagnostics().iter().map(|d| d.to_string()).collect();
    (report.is_clean(), lines)
}

#[test]
fn valid_files_have_no_errors_and_exactly_the_stated_warnings() {
    for path in corpus("tests/conformance/valid") {
        let (clean, got) = diagnostics(&path);
        assert!(clean, "{} has errors: {got:?}", path.display());
        assert_eq!(got, expected(&path), "{}", path.display());
    }
}

#[test]
fn invalid_files_have_errors_and_exactly_the_stated_diagnostics() {
    for path in corpus("tests/conformance/invalid") {
        let (clean, got) = diagnostics(&path);
        assert!(!clean, "{} should have errors", path.display());
        let want = expected(&path);
        assert!(
            !want.is_empty(),
            "{} has no .expected sidecar",
            path.display()
        );
        assert_eq!(got, want, "{}", path.display());
    }
}

#[test]
fn the_spec_is_itself_a_clean_literate_document() {
    let path = Path::new("docs/spec/cola.md");
    let (clean, got) = diagnostics(path);
    assert!(clean && got.is_empty(), "{}: {got:?}", path.display());
}

// ---- Token-level parity: the highlighter's identifier, number and
// string tokens on every cola line are exactly the parser's located
// terminals. ----

type Term = (u32, usize, usize, &'static str);

/// Keywords are keywords to the highlighter wherever they stand; the
/// parser accepts them as names by position (see the spec). Such names
/// are left out of the comparison on both sides.
fn is_keyword(text: &str) -> bool {
    matches!(text, "plural" | "true" | "false")
}

struct Walk<'a> {
    body: Vec<&'a str>,
    offset: u32,
    terms: BTreeSet<Term>,
    /// File lines inside a multi-line string, which a line lexer cannot see as one token.
    skip: BTreeSet<u32>,
}

impl Walk<'_> {
    fn term(&mut self, v: &ValLoc<String>, kind: &'static str) {
        let loc = v.location.expect("terminals carry locations");
        let end = loc.end.expect("terminals carry an end");
        let (line, start) = (loc.start.line() as u32, loc.start.column());
        if end.line() as u32 != line {
            for l in line..=end.line() as u32 {
                self.skip.insert(l + self.offset);
            }
            return;
        }
        let text = self.body[line as usize - 1];
        assert_eq!(
            &text[start..end.column()],
            v.as_ref(),
            "rustemo location does not match the text on line {line}"
        );
        if !is_keyword(v.as_ref()) {
            self.terms
                .insert((line + self.offset, start, end.column(), kind));
        }
    }

    fn entity(&mut self, e: &Entity) {
        match e {
            Entity::SingularEntity(s) => {
                self.term(&s.identifier, "Identifier");
                self.definition(&s.entity_definition);
            }
            Entity::PluralEntity(p) => {
                self.term(&p.identifier_1, "Identifier");
                self.term(&p.identifier_3, "Identifier");
                self.definition(&p.entity_definition);
            }
        }
    }

    fn definition(&mut self, d: &EntityDefinition) {
        for block in d.iter().flatten() {
            match block {
                NestedBlock::FieldList(fl) => self.fields(fl),
                NestedBlock::Entity(e) => self.entity(e),
            }
        }
    }

    fn fields(&mut self, fl: &FieldList) {
        match fl {
            FieldList::Field(f) => self.field(f),
            FieldList::C2(c) => {
                self.fields(&c.field_list);
                self.field(&c.field);
            }
        }
    }

    fn field(&mut self, f: &Field) {
        self.term(&f.identifier, "Identifier");
        match &f.field_value {
            FieldValue::QuotedStringDouble(s) | FieldValue::QuotedStringSingle(s) => {
                self.term(s, "String")
            }
            FieldValue::Number(n) => self.term(n, "Number"),
            FieldValue::BooleanTrue | FieldValue::BooleanFalse => {}
        }
    }
}

/// The parser's located terminals in a file, and the lines to skip.
fn parser_terms(text: &str) -> (BTreeSet<Term>, BTreeSet<u32>) {
    let mut terms = BTreeSet::new();
    let mut skip = BTreeSet::new();
    for block in scan(text).cola_blocks() {
        let Ok(ast): Result<Cola, _> = ColaParser::new().parse(&block.body) else {
            // An invalid block has no terminals to compare: leave its
            // lines out on the highlighter's side too.
            let lines = block.body.split_inclusive('\n').count() as u32;
            skip.extend(block.body_line()..block.body_line() + lines);
            continue;
        };
        let mut walk = Walk {
            body: block.body.split_inclusive('\n').collect(),
            offset: block.body_line() - 1,
            terms: BTreeSet::new(),
            skip: BTreeSet::new(),
        };
        if let Some(entities) = &ast {
            for e in entities {
                walk.entity(e);
            }
        }
        terms.extend(walk.terms);
        skip.extend(walk.skip);
    }
    // A line that holds part of a multi-line string is left out on both
    // sides, including the terminals that precede the string on it.
    terms.retain(|t| !skip.contains(&t.0));
    (terms, skip)
}

/// The highlighter's identifier, number and string tokens on cola lines.
fn highlighter_terms(text: &str, skip: &BTreeSet<u32>) -> BTreeSet<Term> {
    let mut out = BTreeSet::new();
    let mut state = LineState::default();
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let file_line = index as u32 + 1;
        let (tokens, next) = line_tokens(line, state);
        if state.is_cola() && next.is_cola() && !skip.contains(&file_line) {
            for t in tokens {
                let kind = match t.kind {
                    TokenKind::Identifier => "Identifier",
                    TokenKind::Number => "Number",
                    TokenKind::String => "String",
                    _ => continue,
                };
                out.insert((file_line, t.range.start, t.range.end, kind));
            }
        }
        state = next;
    }
    out
}

#[test]
fn highlighter_tokens_are_the_parsers_terminals() {
    let mut files = corpus("tests/conformance/valid");
    files.extend(corpus("tests/conformance/invalid"));
    files.extend(corpus("tests/data"));
    files.extend(corpus("tests/data/fmt"));
    files.push(PathBuf::from("docs/spec/cola.md"));
    let mut compared = 0;
    for path in files {
        if path.extension().is_some_and(|e| e == "cola") {
            continue; // bare files have no fences; the line lexer starts in prose
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let (parser, skip) = parser_terms(&text);
        let highlighter = highlighter_terms(&text, &skip);
        assert_eq!(highlighter, parser, "{}", path.display());
        compared += parser.len();
    }
    assert!(compared > 200, "only {compared} terminals compared");
}
