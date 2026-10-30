// SPDX-License-Identifier: Apache-2.0
//! The literal forms of values: decoding what the grammar matched into
//! the model's values, and encoding values back into canonical text.
//! The two are inverses, and the block-string encoder checks that
//! they are before it commits to the block form.

use crate::model::config_model::ConfigValue;

/// The text between the quotes of `"…"` or `'…'`, with its escapes
/// decoded: `\"`, `\'`, `\\`, `\n`, `\r`, `\t` and `\u{…}` with one to
/// six hex digits. Any other escape is an error naming it.
pub fn decode_quoted(raw: &str) -> Result<String, String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let rest = chars.as_str();
                let hex = rest
                    .strip_prefix('{')
                    .and_then(|r| r.split_once('}'))
                    .filter(|(hex, _)| (1..=6).contains(&hex.len()))
                    .and_then(|(hex, _)| u32::from_str_radix(hex, 16).ok().map(|n| (hex, n)));
                match hex.and_then(|(hex, n)| char::from_u32(n).map(|c| (hex, c))) {
                    Some((hex, decoded)) => {
                        out.push(decoded);
                        chars = rest[hex.len() + 2..].chars();
                    }
                    None => {
                        let shown: String = rest.chars().take(8).collect();
                        return Err(format!("invalid unicode escape `\\u{shown}`"));
                    }
                }
            }
            Some(other) => return Err(format!("unknown escape `\\{other}`")),
            // The lexer never ends a string on a lone backslash.
            None => return Err("unknown escape `\\`".into()),
        }
    }
    Ok(out)
}

fn leading_whitespace(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

fn is_blank(line: &str) -> bool {
    line.chars().all(|c| c == ' ' || c == '\t' || c == '\r')
}

/// The text between the delimiters of a `"""…"""` block string: the
/// first line is dropped when blank, the last line is dropped with the
/// newline before it when blank, and the longest common leading
/// whitespace of the non-blank lines is removed from every line. No
/// escapes are decoded.
pub fn decode_block(raw: &str) -> String {
    let mut text = raw;
    if let Some(nl) = text.find('\n')
        && is_blank(&text[..nl])
    {
        text = &text[nl + 1..];
    }
    if let Some(nl) = text.rfind('\n')
        && is_blank(&text[nl + 1..])
    {
        text = &text[..nl];
    }
    let common = text
        .split('\n')
        .filter(|line| !is_blank(line))
        .map(leading_whitespace)
        .min()
        .unwrap_or(0);
    text.split('\n')
        .map(|line| {
            let strip = leading_whitespace(line).min(common);
            &line[line
                .char_indices()
                .nth(strip)
                .map_or(line.len(), |(i, _)| i)..]
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A number literal as the grammar matched it: digit separators are
/// dropped; a decimal point or an exponent makes a float.
pub fn parse_number(text: &str) -> Result<ConfigValue, String> {
    let digits: String = text.chars().filter(|c| *c != '_').collect();
    if digits.contains(['.', 'e', 'E']) {
        digits
            .parse::<f64>()
            .map(ConfigValue::Float)
            .map_err(|_| format!("invalid float: {text}"))
    } else {
        digits
            .parse::<i64>()
            .map(ConfigValue::Integer)
            .map_err(|_| format!("integer out of range: {text}"))
    }
}

/// A float as cola text: always with a decimal point, so `8.0` stays a
/// float, and in exponent form when the plain decimal would run past
/// twenty characters, so `1e300` does not become three hundred digits.
pub fn float_text(value: f64) -> String {
    let plain = format!("{value}");
    let text = if plain.len() > 20 && value.is_finite() {
        let exp = format!("{value:e}");
        match exp.split_once('e') {
            Some((mantissa, exponent)) if !mantissa.contains('.') => {
                format!("{mantissa}.0e{exponent}")
            }
            _ => exp,
        }
    } else {
        plain
    };
    if text.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        format!("{text}.0")
    } else {
        text
    }
}

fn escape_into(out: &mut String, c: char, quote: char) {
    match c {
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        c if c == quote => {
            out.push('\\');
            out.push(c);
        }
        c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
        c => out.push(c),
    }
}

/// A string as a one-line cola literal: double-quoted, unless the text
/// contains a double quote and no single quote, then single-quoted, so
/// that the quote inside needs no escape.
pub fn quote(text: &str) -> String {
    let q = if text.contains('"') && !text.contains('\'') {
        '\''
    } else {
        '"'
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(q);
    for c in text.chars() {
        escape_into(&mut out, c, q);
    }
    out.push(q);
    out
}

/// A string as a block literal whose closing `"""` stands at `indent`
/// and whose lines are indented one level deeper, or `None` when the
/// block form would not read back as exactly `text`.
pub fn block(text: &str, indent: &str, level: &str) -> Option<String> {
    if !text.contains('\n') || text.contains("\"\"\"") {
        return None;
    }
    let inner = format!("{level}{indent}");
    let mut out = String::from("\"\"\"\n");
    let mut raw = String::from("\n");
    for line in text.split('\n') {
        let written = if line.is_empty() {
            String::new()
        } else {
            format!("{inner}{line}")
        };
        out.push_str(&written);
        out.push('\n');
        raw.push_str(&written);
        raw.push('\n');
    }
    out.push_str(indent);
    out.push_str("\"\"\"");
    raw.push_str(indent);
    (decode_block(&raw) == text).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_escapes_decode() {
        assert_eq!(decode_quoted(r#"say \"hi\""#).unwrap(), "say \"hi\"");
        assert_eq!(decode_quoted(r"it\'s").unwrap(), "it's");
        assert_eq!(decode_quoted(r"a\\b\n\t\r").unwrap(), "a\\b\n\t\r");
        assert_eq!(decode_quoted(r"\u{e9}\u{1F600}").unwrap(), "é😀");
        assert_eq!(decode_quoted(r"\q").unwrap_err(), "unknown escape `\\q`");
        assert_eq!(
            decode_quoted(r"\u{zz}").unwrap_err(),
            "invalid unicode escape `\\u{zz}`"
        );
        assert!(decode_quoted(r"\u{110000}").is_err());
        assert!(decode_quoted("\\u1234").is_err());
    }

    #[test]
    fn block_strings_dedent() {
        assert_eq!(decode_block("\n    a\n      b\n    "), "a\n  b");
        assert_eq!(decode_block("\n    a\n    b\n\n    "), "a\nb\n");
        assert_eq!(decode_block("one line"), "one line");
        assert_eq!(decode_block("  x  "), "x  ");
        assert_eq!(decode_block("\n\ta\n\t\tb\n"), "a\n\tb");
        // A blank line shorter than the common indent is emptied, a
        // longer one keeps its surplus.
        assert_eq!(decode_block("\n    a\n  \n      \n    b\n"), "a\n\n  \nb");
    }

    #[test]
    fn numbers() {
        assert!(matches!(
            parse_number("1_000"),
            Ok(ConfigValue::Integer(1000))
        ));
        assert!(matches!(parse_number("-7"), Ok(ConfigValue::Integer(-7))));
        assert!(matches!(parse_number("1e3"), Ok(ConfigValue::Float(f)) if f == 1000.0));
        assert!(matches!(parse_number("1.5E-3"), Ok(ConfigValue::Float(f)) if f == 0.0015));
        assert!(matches!(parse_number("1_0.0_1"), Ok(ConfigValue::Float(f)) if f == 10.01));
        assert_eq!(
            parse_number("99999999999999999999").unwrap_err(),
            "integer out of range: 99999999999999999999"
        );
    }

    #[test]
    fn floats_keep_their_point_and_long_ones_use_exponents() {
        assert_eq!(float_text(8.0), "8.0");
        assert_eq!(float_text(-3.0), "-3.0");
        assert_eq!(float_text(0.5), "0.5");
        assert_eq!(float_text(1048576.25), "1048576.25");
        assert_eq!(float_text(1e300), "1.0e300");
        assert_eq!(float_text(1.5e-300), "1.5e-300");
        assert_eq!(float_text(-2.5e25), "-2.5e25");
        assert_eq!(float_text(12345678901234567890.0), "12345678901234567000.0");
        assert_eq!(float_text(1e21), "1.0e21");
    }

    #[test]
    fn quoting_escapes_and_picks_the_quote() {
        assert_eq!(quote("plain"), "\"plain\"");
        assert_eq!(quote("say \"hi\""), "'say \"hi\"'");
        assert_eq!(quote("it's"), "\"it's\"");
        assert_eq!(quote("it's \"x\""), "\"it's \\\"x\\\"\"");
        assert_eq!(quote("a\\b\n"), "\"a\\\\b\\n\"");
        assert_eq!(quote("bell\u{7}"), "\"bell\\u{7}\"");
        for text in ["", "a'b\"c\\", "\u{1}\t\r", "é😀"] {
            let q = quote(text);
            let inner = &q[1..q.len() - 1];
            assert_eq!(decode_quoted(inner).unwrap(), text, "{q}");
        }
    }

    #[test]
    fn block_form_round_trips_or_declines() {
        assert_eq!(
            block("a\n  b", "    ", "    ").unwrap(),
            "\"\"\"\n        a\n          b\n    \"\"\""
        );
        assert_eq!(
            block("a\nb\n", "", "    ").unwrap(),
            "\"\"\"\n    a\n    b\n\n\"\"\""
        );
        assert!(block("one line", "", "    ").is_none());
        assert!(block("has \"\"\" inside\n", "", "    ").is_none());
        // Leading whitespace on the first line would be taken as indent.
        assert!(block("  a\nb", "", "    ").is_some());
        assert!(block("a\n b", "", "    ").is_some());
        assert!(block("\na", "", "    ").is_some());
    }
}
