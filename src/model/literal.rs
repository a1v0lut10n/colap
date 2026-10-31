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

/// An exact decimal number, kept as written.
///
/// The text is the literal in canonical spelling: digit separators, a
/// leading `+` and leading zeros of the integer part dropped, the
/// exponent letter in lower case, nothing else changed. The value is exactly what the literal writes, of any
/// size; equality, order and hashing are numeric, so `1.10`, `1.1` and
/// `11e-1` are one value spelled three ways. The literal's kind,
/// integer or float, is kept beside the value for the schema's types
/// and the generator's inference. See
/// `docs/design/2026-10-31-exact-numbers.md`.
#[derive(Debug, Clone)]
pub struct Number {
    text: String,
}

/// A number in normal form: sign, significand digits without leading
/// or trailing zeros (`"0"` for zero, never negative), and the power
/// of ten that scales them. Equal numbers have equal normal forms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Normal {
    negative: bool,
    digits: String,
    exponent: i64,
}

impl Number {
    /// The number a grammar-matched literal writes.
    pub fn parse(literal: &str) -> Result<Number, String> {
        let cleaned: String = literal
            .trim()
            .trim_start_matches('+')
            .chars()
            .filter(|c| *c != '_')
            .map(|c| if c == 'E' { 'e' } else { c })
            .collect();
        let negative = cleaned.starts_with('-');
        let body = cleaned.strip_prefix('-').unwrap_or(&cleaned);
        let (mantissa, exponent) = body.split_once('e').unwrap_or((body, "0"));
        let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, "0"));
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let exponent_ok = digits(exponent.trim_start_matches(['+', '-']))
            && exponent
                .trim_start_matches(['+', '-'])
                .parse::<i64>()
                .is_ok();
        if !digits(int) || !digits(frac) || !exponent_ok {
            return Err(format!("invalid number: {literal}"));
        }
        // Leading zeros of the integer part say nothing; one digit stays.
        let int = int.trim_start_matches('0');
        let int = if int.is_empty() { "0" } else { int };
        let mut text = String::with_capacity(cleaned.len());
        if negative {
            text.push('-');
        }
        text.push_str(int);
        if mantissa.contains('.') {
            text.push('.');
            text.push_str(frac);
        }
        if body.contains('e') {
            text.push('e');
            text.push_str(exponent);
        }
        Ok(Number { text })
    }

    /// The canonical literal.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Whether the literal is of integer kind: no decimal point and no
    /// exponent. `8` is, `8.0` and `8e0` are not, though all three are
    /// the same value.
    pub fn is_integer(&self) -> bool {
        !self.text.contains(['.', 'e'])
    }

    pub fn is_negative(&self) -> bool {
        self.text.starts_with('-') && self.normal().digits != "0"
    }

    /// The value as `±digits × 10^exponent`: the sign, the significand
    /// as a decimal digit string with no leading or trailing zeros
    /// (`"0"` for zero), and the power of ten. A bignum consumer reads
    /// this without loss.
    pub fn decimal(&self) -> (bool, String, i64) {
        let n = self.normal();
        (n.negative, n.digits, n.exponent)
    }

    /// The value as an `i64`, when it is integral and fits, whatever
    /// its kind: `8.0` and `1e3` qualify, `8.5` and `1e30` do not.
    pub fn as_i64(&self) -> Option<i64> {
        let n = self.normal();
        if n.exponent < 0 {
            return None;
        }
        // Accumulated with the sign applied, so that `i64::MIN` fits.
        let digit = |b: u8| {
            let d = i64::from(b - b'0');
            if n.negative { -d } else { d }
        };
        let mut value: i64 = 0;
        for b in n.digits.bytes() {
            value = value.checked_mul(10)?.checked_add(digit(b))?;
        }
        for _ in 0..n.exponent {
            value = value.checked_mul(10)?;
        }
        Some(value)
    }

    /// The nearest `f64`, correctly rounded; infinite past its range.
    pub fn as_f64(&self) -> f64 {
        self.text.parse().unwrap_or(f64::NAN)
    }

    fn normal(&self) -> Normal {
        let body = self.text.strip_prefix('-').unwrap_or(&self.text);
        let negative = body.len() != self.text.len();
        let (mantissa, exponent) = body.split_once('e').unwrap_or((body, "0"));
        let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let mut exponent: i64 = exponent.parse().unwrap_or(0);
        let mut digits = format!("{int}{frac}");
        exponent -= frac.len() as i64;
        let trailing = digits.len() - digits.trim_end_matches('0').len();
        digits.truncate(digits.len() - trailing);
        exponent += trailing as i64;
        let digits = digits.trim_start_matches('0').to_string();
        if digits.is_empty() {
            return Normal {
                negative: false,
                digits: "0".into(),
                exponent: 0,
            };
        }
        Normal {
            negative,
            digits,
            exponent,
        }
    }
}

/// Zero.
impl Default for Number {
    fn default() -> Self {
        Number { text: "0".into() }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.normal() == other.normal()
    }
}

impl Eq for Number {}

impl std::hash::Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normal().hash(state);
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let (a, b) = (self.normal(), other.normal());
        match (a.negative, b.negative) {
            (false, true) => return Ordering::Greater,
            (true, false) => return Ordering::Less,
            _ => {}
        }
        let magnitude = |n: &Normal| {
            if n.digits == "0" {
                i64::MIN
            } else {
                n.digits.len() as i64 + n.exponent
            }
        };
        // Same sign: compare magnitudes, the position of the leading
        // digit first, then the digits padded to one length.
        let by_magnitude = magnitude(&a).cmp(&magnitude(&b)).then_with(|| {
            let width = a.digits.len().max(b.digits.len());
            let pad = |d: &str| format!("{d:0<width$}");
            pad(&a.digits).cmp(&pad(&b.digits))
        });
        if a.negative {
            by_magnitude.reverse()
        } else {
            by_magnitude
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number {
            text: value.to_string(),
        }
    }
}

/// The shortest literal that reads back as the same double, always of
/// float kind (`8.0`), in exponent form when the plain decimal would
/// run past twenty characters (`1e300`). Not for infinities or NaN,
/// which no literal writes: they become `0.0`.
impl From<f64> for Number {
    fn from(value: f64) -> Self {
        if !value.is_finite() {
            return Number { text: "0.0".into() };
        }
        let plain = format!("{value}");
        let text = if plain.len() > 20 {
            format!("{value:e}")
        } else if plain.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            format!("{plain}.0")
        } else {
            plain
        };
        Number { text }
    }
}

/// A number literal as the grammar matched it.
pub fn parse_number(text: &str) -> Result<ConfigValue, String> {
    Number::parse(text).map(ConfigValue::Number)
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
    fn numbers_keep_their_spelling_and_compare_by_value() {
        let n = |t: &str| Number::parse(t).unwrap();
        assert_eq!(n("1_000").text(), "1000");
        assert_eq!(n("+7").text(), "7");
        assert_eq!(n("1.5E-3").text(), "1.5e-3");
        assert_eq!(n("1.10").text(), "1.10");
        assert_eq!(n("1_0.0_1").text(), "10.01");
        assert_eq!(n("007").text(), "7");
        assert_eq!(n("00.50").text(), "0.50");
        assert_eq!(n("-0").text(), "-0");
        assert_eq!(n("1e+3").text(), "1e+3");
        assert_eq!(n("1.10"), n("1.1"));
        assert_eq!(n("1.1"), n("11e-1"));
        assert_eq!(n("0.0"), n("-0"));
        assert_eq!(n("1e3"), n("1000"));
        assert!(n("8").is_integer() && !n("8.0").is_integer() && !n("8e0").is_integer());
        assert!(n("-1e-3").is_negative() && !n("-0.0").is_negative());
        assert!(Number::parse("1.").is_err() && Number::parse("e3").is_err());
        let mut sorted: Vec<Number> = ["1e3", "-2", "0.5", "10", "-0.25", "999.9", "0", "-1e2"]
            .map(n)
            .into();
        sorted.sort();
        let texts: Vec<&str> = sorted.iter().map(Number::text).collect();
        assert_eq!(
            texts,
            ["-1e2", "-2", "-0.25", "0", "0.5", "10", "999.9", "1e3"]
        );
    }

    #[test]
    fn number_views() {
        let n = |t: &str| Number::parse(t).unwrap();
        assert_eq!(n("8.0").as_i64(), Some(8));
        assert_eq!(n("1e3").as_i64(), Some(1000));
        assert_eq!(n("-9223372036854775808").as_i64(), Some(i64::MIN));
        assert_eq!(n("9223372036854775808").as_i64(), None);
        assert_eq!(n("8.5").as_i64(), None);
        assert_eq!(n("1e30").as_i64(), None);
        assert_eq!(n("0.1").as_f64(), 0.1);
        assert_eq!(n("1e400").as_f64(), f64::INFINITY);
        assert_eq!(
            n("123456789012345678901234567890").decimal(),
            (false, "12345678901234567890123456789".into(), 1)
        );
        assert_eq!(n("-0.0150").decimal(), (true, "15".into(), -3));
        assert_eq!(n("0e5").decimal(), (false, "0".into(), 0));
        assert_eq!(Number::from(42i64).text(), "42");
        assert_eq!(Number::from(8.0f64).text(), "8.0");
        assert_eq!(Number::from(1e300f64).text(), "1e300");
        assert_eq!(Number::from(1.5e-300f64).text(), "1.5e-300");
        assert_eq!(
            Number::from(12345678901234567890.0f64).text(),
            "12345678901234567000.0"
        );
        assert_eq!(Number::from(1e21f64).text(), "1e21");
        assert_eq!(Number::from(f64::INFINITY).text(), "0.0");
        assert_eq!(Number::from(-2.5f64).text(), "-2.5");
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
