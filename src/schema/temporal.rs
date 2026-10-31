// SPDX-License-Identifier: Apache-2.0
//! The temporal types: what a string must look like to be an
//! `instant`, a `date`, a `datetime` or an `edtf` value
//! (`docs/design/2026-10-31-schema-dialect.md`, "Time"). Instants are
//! transaction time and carry an offset; the other three are civil
//! valid time and never do.

use chrono::NaiveDate;

/// An RFC 3339 date-time with `Z` or an offset, naming a real instant.
/// Returns whether it is written in UTC (`Z`), for the `_utc` warning.
pub fn instant(text: &str) -> Option<bool> {
    chrono::DateTime::parse_from_rfc3339(text).ok()?;
    Some(text.ends_with(['Z', 'z']))
}

fn digits(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
}

fn number(s: &str) -> Option<u32> {
    s.parse().ok()
}

/// A calendar day exists.
fn valid_day(year: i32, month: u32, day: u32) -> bool {
    NaiveDate::from_ymd_opt(year, month, day).is_some()
}

/// `YYYY`, `YYYY-MM` or `YYYY-MM-DD`, a real date at its precision.
pub fn date(text: &str) -> bool {
    let mut parts = text.split('-');
    let (Some(y), m, d, None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if !digits(y, 4) {
        return false;
    }
    let Some(year) = y.parse::<i32>().ok() else {
        return false;
    };
    match (m, d) {
        (None, None) => true,
        (Some(m), None) => digits(m, 2) && number(m).is_some_and(|m| (1..=12).contains(&m)),
        (Some(m), Some(d)) => {
            digits(m, 2)
                && digits(d, 2)
                && match (number(m), number(d)) {
                    (Some(m), Some(d)) => valid_day(year, m, d),
                    _ => false,
                }
        }
        _ => false,
    }
}

/// `hh:mm`, `hh:mm:ss` or `hh:mm:ss.fff…`.
fn time(text: &str) -> bool {
    let (hms, fraction) = match text.split_once('.') {
        Some((hms, f)) => (hms, Some(f)),
        None => (text, None),
    };
    let parts: Vec<&str> = hms.split(':').collect();
    let ok = match parts.as_slice() {
        [h, m] => digits(h, 2) && digits(m, 2),
        [h, m, s] => digits(h, 2) && digits(m, 2) && digits(s, 2),
        _ => false,
    } && number(parts[0]).is_some_and(|h| h < 24)
        && number(parts[1]).is_some_and(|m| m < 60)
        && parts
            .get(2)
            .is_none_or(|s| number(s).is_some_and(|s| s < 60));
    ok && fraction
        .is_none_or(|f| !f.is_empty() && parts.len() == 3 && f.bytes().all(|b| b.is_ascii_digit()))
}

/// An IANA zone identifier's shape, `Area/Location` with one or more
/// segments after the area; existence is not checked.
fn zone(text: &str) -> bool {
    let mut segments = text.split('/');
    let area = segments.next().unwrap_or("");
    let rest: Vec<&str> = segments.collect();
    let segment = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'+'))
    };
    area.bytes().all(|b| b.is_ascii_alphabetic())
        && !area.is_empty()
        && !rest.is_empty()
        && rest.iter().all(|s| segment(s))
}

/// A civil date and time with no offset: `YYYY-MM-DDThh:mm`, seconds
/// and a fraction optional, and optionally `[Area/Location]` after it
/// as RFC 9557 writes a zoned civil time.
pub fn datetime(text: &str) -> bool {
    let (stamp, zone_part) = match text.split_once('[') {
        Some((stamp, rest)) => match rest.strip_suffix(']') {
            Some(z) => (stamp, Some(z)),
            None => return false,
        },
        None => (text, None),
    };
    let Some((d, t)) = stamp.split_once('T') else {
        return false;
    };
    d.len() == 10 && date(d) && time(t) && zone_part.is_none_or(zone)
}

// ---- EDTF, levels 0 and 1 ----

/// A year: four digits, or `Y` and five or more, either with a sign;
/// level 1 lets trailing digits be `X`. Returns the year when it has
/// no `X`, for calendar checks.
fn edtf_year(text: &str) -> Option<Option<i32>> {
    let (long, body) = match text.strip_prefix('Y') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let unsigned = body.strip_prefix('-').unwrap_or(body);
    let ok_len = if long {
        unsigned.len() >= 5
    } else {
        unsigned.len() == 4
    };
    if !ok_len || unsigned.is_empty() {
        return None;
    }
    let specified = unsigned.trim_end_matches('X');
    if !specified.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if specified.len() == unsigned.len() {
        Some(body.parse().ok())
    } else {
        Some(None)
    }
}

/// A two-digit component, or `XX`.
fn edtf_component(text: &str) -> Option<Option<u32>> {
    if text == "XX" {
        Some(None)
    } else if digits(text, 2) {
        Some(number(text))
    } else {
        None
    }
}

/// A level 0 or 1 date: year, month or season, day, with `X` for
/// unspecified digits and a trailing `?`, `~` or `%` qualifier.
fn edtf_date(text: &str) -> bool {
    let body = text.trim_end_matches(['?', '~', '%']);
    if text.len() - body.len() > 1 {
        return false;
    }
    // A `Y` prefix and a negative sign are part of the year, not
    // separators.
    let (prefix, rest) = ["Y-", "Y", "-"]
        .into_iter()
        .find_map(|p| body.strip_prefix(p).map(|rest| (p, rest)))
        .unwrap_or(("", body));
    let mut parts = rest.split('-');
    let (Some(y), m, d, None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let Some(year) = edtf_year(&format!("{prefix}{y}")) else {
        return false;
    };
    match (m, d) {
        (None, None) => true,
        (Some(m), None) => match edtf_component(m) {
            Some(Some(m)) => (1..=12).contains(&m) || (21..=24).contains(&m),
            Some(None) => true,
            None => false,
        },
        (Some(m), Some(d)) => match (edtf_component(m), edtf_component(d)) {
            (Some(Some(m)), Some(Some(d))) => match year {
                Some(year) => valid_day(year, m, d),
                None => (1..=12).contains(&m) && (1..=31).contains(&d),
            },
            (Some(Some(m)), Some(None)) => (1..=12).contains(&m),
            (Some(None), Some(d)) => d.is_none_or(|d| (1..=31).contains(&d)),
            _ => false,
        },
        _ => false,
    }
}

/// A level 0 date-time: a day-precision date, `T`, a time, and an
/// optional `Z` or offset.
fn edtf_datetime(text: &str) -> bool {
    let Some((d, t)) = text.split_once('T') else {
        return false;
    };
    let t = t.strip_suffix('Z').unwrap_or(t);
    let t = match t.rfind(['+', '-']) {
        Some(i) => {
            let (time_part, offset) = t.split_at(i);
            match offset[1..].split_once(':') {
                Some((h, m)) if digits(h, 2) && digits(m, 2) => time_part,
                _ => return false,
            }
        }
        None => t,
    };
    d.len() == 10 && date(d) && time(t)
}

/// One side of an interval: a date, `..` for open, or nothing for
/// unknown.
fn edtf_end(text: &str) -> bool {
    text.is_empty() || text == ".." || edtf_date(text)
}

/// An EDTF level 0 or level 1 value: a date, a date-time, or an
/// interval of two dates with `..` or an empty side for an open or
/// unknown end.
pub fn edtf(text: &str) -> bool {
    if text.contains('T') {
        return edtf_datetime(text);
    }
    match text.split_once('/') {
        Some((a, b)) => !(a.is_empty() && b.is_empty()) && edtf_end(a) && edtf_end(b),
        None => edtf_date(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instants_need_an_offset_and_a_real_moment() {
        assert_eq!(instant("2026-10-30T14:05:00Z"), Some(true));
        assert_eq!(instant("2026-10-30T14:05:00.5+02:00"), Some(false));
        assert_eq!(instant("2026-10-30T14:05:00+00:00"), Some(false));
        assert_eq!(instant("2026-02-30T00:00:00Z"), None);
        assert_eq!(instant("2026-10-30T14:05"), None);
        assert_eq!(instant("2026-10-30"), None);
    }

    #[test]
    fn dates_at_three_precisions() {
        for ok in ["1914", "1914-08", "1914-08-04", "2024-02-29", "0001-01-01"] {
            assert!(date(ok), "{ok}");
        }
        for bad in [
            "1914-8",
            "1914-13",
            "1914-08-32",
            "2023-02-29",
            "19140804",
            "1914-08-04T10:00",
            "-1914",
            "914",
        ] {
            assert!(!date(bad), "{bad}");
        }
    }

    #[test]
    fn datetimes_are_civil_and_may_carry_a_zone() {
        for ok in [
            "1919-06-28T15:12",
            "1919-06-28T15:12:00",
            "1919-06-28T15:12:00.250",
            "2026-10-30T09:00[Europe/Zurich]",
            "2026-10-30T09:00:00[America/Argentina/Buenos_Aires]",
            "2026-10-30T09:00[Etc/GMT+1]",
        ] {
            assert!(datetime(ok), "{ok}");
        }
        for bad in [
            "1919-06-28",
            "1919-06-28T24:00",
            "1919-06-28T15:60",
            "1919-06-28T15:12Z",
            "1919-06-28T15:12+02:00",
            "1919-06-28T15:12.5",
            "2026-10-30T09:00[Zurich]",
            "2026-10-30T09:00[CET]",
            "2026-10-30T09:00[Europe/Zurich",
        ] {
            assert!(!datetime(bad), "{bad}");
        }
    }

    #[test]
    fn edtf_levels_0_and_1() {
        for ok in [
            "1914",
            "1914-08",
            "1914-08-04",
            "1914?",
            "1850~",
            "1850%",
            "191X",
            "19XX",
            "1914-XX",
            "1914-08-XX",
            "1914-XX-XX",
            "Y170000002",
            "Y-17000",
            "-0999",
            "2001-21",
            "1914/1918",
            "1914/..",
            "../1918",
            "1914/",
            "/1918",
            "1914-08-04/1918-11-11",
            "1985-04-12T23:20:30",
            "1985-04-12T23:20:30Z",
            "1985-04-12T23:20:30+04:00",
        ] {
            assert!(edtf(ok), "{ok}");
        }
        for bad in [
            "",
            "/",
            "19",
            "1914-13",
            "1914-25",
            "2023-02-29",
            "1914??",
            "X914",
            "1914-X8",
            "Y1914",
            "1914-08-04T25:00:00",
            "1914/1918/1920",
            "1914-08-04T10:00+4",
        ] {
            assert!(!edtf(bad), "{bad}");
        }
    }
}
