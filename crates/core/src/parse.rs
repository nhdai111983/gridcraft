//! Parsing what people type into a cell (and numeric text in formulas).
//!
//! Recognises numbers with thousands separators, percentages, currency, scientific notation,
//! fractions (`1 1/2`), dates (`3/4/2026`, `2026-03-04`, `4-Mar-2026`, `Mar 4, 2026`), times
//! (`10:30`, `10:30:15 PM`) and date-times. Locale: en-US; [`parse_input_in`] reads the other
//! regional formats ([`Locale`]) for the interface.

use crate::date::{DateSystem, MONTHS, serial_from_ymd, time_fraction};
use crate::locale::{DONG, Locale};
use crate::value::{CellError, Value};

/// A parsed entry: its value and the number format Excel would apply automatically.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub value: Value,
    pub format: Option<&'static str>,
}

/// Interprets typed cell input (not formulas). Text that isn't a number, date, boolean or error
/// stays text. A leading apostrophe forces text.
pub fn parse_input(s: &str, sys: DateSystem) -> Parsed {
    if s.is_empty() {
        return Parsed { value: Value::Empty, format: None };
    }
    if let Some(rest) = s.strip_prefix('\'') {
        return Parsed { value: Value::text(rest), format: None };
    }
    let t = s.trim();
    if t.eq_ignore_ascii_case("TRUE") {
        return Parsed { value: Value::Bool(true), format: None };
    }
    if t.eq_ignore_ascii_case("FALSE") {
        return Parsed { value: Value::Bool(false), format: None };
    }
    if let Some(e) = CellError::parse(t) {
        return Parsed { value: Value::Error(e), format: None };
    }
    if let Some((n, f)) = parse_numeric(t, sys) {
        return Parsed { value: Value::number(n), format: f };
    }
    Parsed { value: Value::text(s), format: None }
}

/// [`parse_input`] for text typed in a regional format: `1.234,5`, `24/10/2026` and `1.234 ₫` in
/// Vietnamese. The value is the same as the en-US spelling's; so is the automatic number format,
/// which is stored en-US (`#,##0.00`, `m/d/yyyy`) and shown in the locale. en-US is
/// [`parse_input`] itself. Formulas are not parsed here.
pub fn parse_input_in(s: &str, sys: DateSystem, locale: Locale) -> Parsed {
    if locale == Locale::EnUs {
        return parse_input(s, sys);
    }
    if s.is_empty() {
        return Parsed { value: Value::Empty, format: None };
    }
    if let Some(rest) = s.strip_prefix('\'') {
        return Parsed { value: Value::text(rest), format: None };
    }
    let t = s.trim();
    if t.eq_ignore_ascii_case("TRUE") {
        return Parsed { value: Value::Bool(true), format: None };
    }
    if t.eq_ignore_ascii_case("FALSE") {
        return Parsed { value: Value::Bool(false), format: None };
    }
    if let Some(e) = CellError::parse(t) {
        return Parsed { value: Value::Error(e), format: None };
    }
    if let Some((n, f)) = parse_numeric_in(t, sys, locale) {
        return Parsed { value: Value::number(n), format: f };
    }
    Parsed { value: Value::text(s), format: None }
}

/// [`parse_number_text_in`] in a regional format (`VALUE` reads text this way).
pub fn parse_number_text_locale(s: &str, sys: DateSystem, locale: Locale) -> Option<f64> {
    parse_numeric_in(s.trim(), sys, locale).map(|(n, _)| n)
}

/// The currency format a number typed with the đồng sign gets (vi-VN: sign after, no decimals).
pub const VND_FORMAT: &str = "#,##0 [$₫-42A]";
/// [`VND_FORMAT`] for an amount typed with decimals.
pub const VND_FORMAT_DECIMALS: &str = "#,##0.00 [$₫-42A]";

fn parse_numeric_in(t: &str, sys: DateSystem, locale: Locale) -> Option<(f64, Option<&'static str>)> {
    if locale == Locale::EnUs {
        return parse_numeric(t, sys);
    }
    if t.is_empty() {
        return None;
    }
    if let Some(inner) = t.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
        // One pair only: `((1))` is text, as in Excel, and nesting can't grow the stack.
        let inner = inner.trim();
        if inner.starts_with('(') {
            return None;
        }
        return parse_numeric_in(inner, sys, locale).map(|(n, f)| (-n, f));
    }
    // The đồng, after the amount as vi-VN writes it (`1.234 ₫`, `1.234đ`, `1.234 VNĐ`) or before it.
    if let Some(body) = strip_dong(t) {
        let body = body.trim();
        if body.contains(['%', '$', 'e', 'E', DONG]) || strip_dong(body).is_some() || leading_zero_group(body, locale) {
            return None;
        }
        let (n, _) = parse_plain(&locale.delocalize_number(body))?;
        return Some((n, Some(if body.contains(locale.decimal_separator()) { VND_FORMAT_DECIMALS } else { VND_FORMAT })));
    }
    if !leading_zero_group(t, locale)
        && let Some(r) = parse_plain(&locale.delocalize_number(t))
    {
        return Some(r);
    }
    if let Some(r) = parse_fraction(t) {
        return Some((r, Some("# ?/?")));
    }
    parse_datetime_dmy(t, sys)
}

/// The amount of `₫1.234`, `1.234 ₫`, `1.234đ`, `1.234 VNĐ` or `VND 1.234` (case-insensitive).
fn strip_dong(t: &str) -> Option<&str> {
    const SIGNS: [&str; 4] = ["₫", "vnđ", "vnd", "đ"];
    let lower = t.to_lowercase();
    // Lower-casing these letters keeps their byte lengths, so the slices line up.
    if lower.len() != t.len() {
        return None;
    }
    for sign in SIGNS {
        if lower.ends_with(sign) {
            return t.get(..t.len() - sign.len());
        }
        if lower.starts_with(sign) {
            return t.get(sign.len()..);
        }
    }
    None
}

/// `0.125` or `090.312.345` in a locale that groups with `.`: a first group starting with 0 isn't
/// a grouped number (Excel keeps it as text), unlike en-US `0.125`.
fn leading_zero_group(t: &str, locale: Locale) -> bool {
    let body = t.trim_start_matches(['-', '+', ' ', DONG]);
    let int = body.split(locale.decimal_separator()).next().unwrap_or("");
    int.contains(locale.group_separator()) && int.starts_with('0')
}

/// Numeric text as arithmetic coerces it ("1,000", "50%", "$3", "1/2/2020", "10:00").
pub fn parse_number_text(s: &str) -> Option<f64> {
    parse_number_text_in(s, DateSystem::D1900)
}

/// Like [`parse_number_text`], with date text read in the given workbook date system.
pub fn parse_number_text_in(s: &str, sys: DateSystem) -> Option<f64> {
    parse_numeric(s.trim(), sys).map(|(n, _)| n)
}

fn parse_numeric(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)> {
    if t.is_empty() {
        return None;
    }
    // Parenthesised negatives: (100) = -100.
    if let Some(inner) = t.strip_prefix('(').and_then(|x| x.strip_suffix(')')) {
        // One pair only: `((1))` is text, as in Excel, and nesting can't grow the stack.
        let inner = inner.trim();
        if inner.starts_with('(') {
            return None;
        }
        return parse_numeric(inner, sys).map(|(n, f)| (-n, f));
    }
    if let Some(r) = parse_plain(t) {
        return Some(r);
    }
    if let Some(r) = parse_fraction(t) {
        return Some((r, Some("# ?/?")));
    }
    if let Some(r) = parse_datetime(t, sys) {
        return Some(r);
    }
    None
}

/// Signs, currency, thousands separators, percent, exponent.
fn parse_plain(t: &str) -> Option<(f64, Option<&'static str>)> {
    let mut s = t;
    let mut neg = false;
    let mut currency = false;
    let mut percent = false;
    if let Some(r) = s.strip_prefix('-') {
        neg = true;
        s = r.trim_start();
    } else if let Some(r) = s.strip_prefix('+') {
        s = r.trim_start();
    }
    if let Some(r) = s.strip_prefix('$') {
        currency = true;
        s = r.trim_start();
        if !neg && let Some(r2) = s.strip_prefix('-') {
            neg = true;
            s = r2;
        }
    }
    if let Some(r) = s.strip_suffix('%') {
        percent = true;
        s = r.trim_end();
    }
    if s.is_empty() {
        return None;
    }
    let has_sep = s.contains(',');
    let digits: String = if has_sep {
        // Thousands separators must be in groups of three before the decimal point.
        let (int, frac) = s.split_once('.').map_or((s, None), |(a, b)| (a, Some(b)));
        let groups: Vec<&str> = int.split(',').collect();
        let first = groups.first()?;
        if first.is_empty() || first.len() > 3 || groups.iter().skip(1).any(|g| g.len() != 3) {
            return None;
        }
        let mut d = groups.concat();
        if let Some(f) = frac {
            d.push('.');
            d.push_str(f);
        }
        d
    } else {
        s.to_string()
    };
    let b = digits.as_bytes();
    if !b.iter().all(|c| c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-')) {
        return None;
    }
    if !b.first().is_some_and(|c| c.is_ascii_digit() || *c == b'.') {
        return None;
    }
    let mut n: f64 = digits.parse().ok()?;
    if !n.is_finite() {
        return None;
    }
    let sci = digits.contains(['e', 'E']);
    if percent {
        n /= 100.0;
    }
    if neg {
        n = -n;
    }
    let fmt = if currency {
        Some(if digits.contains('.') { "\"$\"#,##0.00" } else { "\"$\"#,##0" })
    } else if percent {
        Some(if digits.contains('.') { "0.00%" } else { "0%" })
    } else if sci {
        Some("0.00E+00")
    } else if has_sep {
        Some(if digits.contains('.') { "#,##0.00" } else { "#,##0" })
    } else {
        None
    };
    Some((n, fmt))
}

/// `1/2` (only with a whole part, else it's a date), `1 1/2`, `0 3/4`.
fn parse_fraction(t: &str) -> Option<f64> {
    let (whole, frac) = t.split_once(' ')?;
    let neg = whole.starts_with('-');
    let w: f64 = whole.trim_start_matches('-').parse().ok()?;
    let (a, b) = frac.trim().split_once('/')?;
    let a: f64 = a.parse().ok()?;
    let b: f64 = b.parse().ok()?;
    if b == 0.0 || !w.is_finite() {
        return None;
    }
    let v = w + a / b;
    Some(if neg { -v } else { v })
}

fn month_from_name(s: &str) -> Option<u32> {
    let l = s.to_ascii_lowercase();
    if l.len() < 3 {
        return None;
    }
    MONTHS.iter().position(|m| m.to_ascii_lowercase().starts_with(&l)).map(|i| i as u32 + 1)
}

/// Time part: `h:mm`, `h:mm:ss`, `h:mm:ss.000`, `h AM`, with optional AM/PM.
fn parse_time(t: &str) -> Option<(f64, bool, bool)> {
    let t = t.trim();
    let lower = t.to_ascii_lowercase();
    let (body, ampm) = if let Some(b) = lower.strip_suffix("am").or_else(|| lower.strip_suffix('a')) {
        (b.trim().to_string(), Some(false))
    } else if let Some(b) = lower.strip_suffix("pm").or_else(|| lower.strip_suffix('p')) {
        (b.trim().to_string(), Some(true))
    } else {
        (lower.clone(), None)
    };
    let parts: Vec<&str> = body.split(':').collect();
    if parts.len() == 1 && ampm.is_none() {
        return None;
    }
    if parts.len() > 3 || parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    let mut h: f64 = parts.first()?.parse().ok()?;
    let m: f64 = parts.get(1).map_or(Some(0.0), |p| p.parse().ok())?;
    let s: f64 = parts.get(2).map_or(Some(0.0), |p| p.parse().ok())?;
    if m >= 60.0 || s >= 60.0 || h < 0.0 || m < 0.0 || s < 0.0 {
        return None;
    }
    if let Some(pm) = ampm {
        if !(0.0..=12.0).contains(&h) {
            return None;
        }
        if h == 12.0 {
            h = 0.0;
        }
        if pm {
            h += 12.0;
        }
    } else if h >= 10000.0 {
        return None;
    }
    Some((time_fraction(h, m, s), parts.len() == 3, ampm.is_some()))
}

fn parse_date(t: &str, sys: DateSystem) -> Option<(f64, &'static str)> {
    let t = t.trim().trim_end_matches(',');
    let num = |s: &str| s.trim().parse::<i64>().ok();
    // ISO yyyy-mm-dd or yyyy/mm/dd
    let seps: &[char] = &['/', '-', '.'];
    let parts: Vec<&str> = t.split(seps).collect();
    if parts.len() == 3 {
        let (a, b, c) = (parts[0].trim(), parts[1].trim(), parts[2].trim());
        if a.len() == 4
            && let (Some(y), Some(m), Some(d)) = (num(a), num(b), num(c))
        {
            return valid_ymd(sys, y, m, d).map(|s| (s, "yyyy-mm-dd"));
        }
        if let (Some(m), Some(d), Some(y)) = (num(a), num(b), num(c)) {
            let y = if c.len() <= 2 { if y < 30 { 2000 + y } else { 1900 + y } } else { y };
            return valid_ymd(sys, y, m, d).map(|s| (s, "m/d/yyyy"));
        }
        // 4-Mar-2026
        if let (Some(d), Some(m), Some(y)) = (num(a), month_from_name(b), num(c)) {
            let y = if c.len() <= 2 { if y < 30 { 2000 + y } else { 1900 + y } } else { y };
            return valid_ymd(sys, y, m as i64, d).map(|s| (s, "d-mmm-yy"));
        }
    }
    if parts.len() == 2 {
        let (a, b) = (parts[0].trim(), parts[1].trim());
        // m/d in the current year is ambiguous without a clock; use month/day of 2026? Excel uses
        // the current year. We accept d-mmm / mmm-d only with month names, and m/d → current year.
        if let (Some(m), Some(d)) = (num(a), num(b)) {
            let y = current_year();
            if (1..=12).contains(&m) && (1..=31).contains(&d) {
                return valid_ymd(sys, y, m, d).map(|s| (s, "d-mmm"));
            }
            // m/yyyy
            if (1..=12).contains(&m) && b.len() == 4 {
                return valid_ymd(sys, d, m, 1).map(|s| (s, "mmm-yy"));
            }
        }
        if let (Some(d), Some(m)) = (num(a), month_from_name(b)) {
            return valid_ymd(sys, current_year(), m as i64, d).map(|s| (s, "d-mmm"));
        }
        if let (Some(m), Some(y)) = (month_from_name(a), num(b)) {
            let y = if b.len() <= 2 { if y < 30 { 2000 + y } else { 1900 + y } } else { y };
            return valid_ymd(sys, y, m as i64, 1).map(|s| (s, "mmm-yy"));
        }
    }
    // "Mar 4, 2026" / "March 4 2026" / "4 March 2026"
    let words: Vec<&str> = t.split([' ', ',']).filter(|w| !w.is_empty()).collect();
    if words.len() == 3 {
        // Two-digit years follow the same 2029 rule as the dashed forms above.
        let expand = |w: &str, y: i64| if w.len() <= 2 { if y < 30 { 2000 + y } else { 1900 + y } } else { y };
        if let (Some(m), Some(d), Some(y)) = (month_from_name(words[0]), num(words[1]), num(words[2])) {
            let y = expand(words[2], y);
            return valid_ymd(sys, y, m as i64, d).map(|s| (s, "d-mmm-yy"));
        }
        if let (Some(d), Some(m), Some(y)) = (num(words[0]), month_from_name(words[1]), num(words[2])) {
            let y = expand(words[2], y);
            return valid_ymd(sys, y, m as i64, d).map(|s| (s, "d-mmm-yy"));
        }
    }
    None
}

fn valid_ymd(sys: DateSystem, y: i64, m: i64, d: i64) -> Option<f64> {
    // Dates before 1900 stay text (`DATE()` would add 1900 to such a year).
    if !(1..=12).contains(&m) || d < 1 || y < 1900 {
        return None;
    }
    let max = crate::date::days_in_month_in(sys, y as i32, m as u32) as i64;
    if d > max {
        return None;
    }
    serial_from_ymd(sys, y, m, d)
}

/// The year used for dates typed without one, from the system clock (browser clock on wasm).
/// Falls back to 2026 only when no clock is available.
pub fn current_year() -> i64 {
    if let Some(ms) = crate::date::unix_now_ms() {
        let days = (ms / 86_400_000.0).floor() as i64;
        return crate::date::civil_from_days(days).0 as i64;
    }
    2026
}

fn time_format(tf: f64, secs: bool, ampm: bool) -> &'static str {
    match (secs, ampm) {
        // 24 hours or more: elapsed hours, so 25:00 shows as 25:00:00, not 1:00.
        _ if tf >= 1.0 => "[h]:mm:ss",
        (false, false) => "h:mm",
        (true, false) => "h:mm:ss",
        (false, true) => "h:mm AM/PM",
        (true, true) => "h:mm:ss AM/PM",
    }
}

/// Day-first dates (`24/10/2026`, `24-10-26`, `24/10`), ISO dates, English month names, times
/// and date-times, as the day-first regional formats type them. `.` is not a date separator
/// there (it groups digits), so `1.5` is not a date.
fn parse_datetime_dmy(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)> {
    let t = &vi_am_pm(t);
    if let Some((tf, secs, ampm)) = parse_time(t) {
        return Some((tf, Some(time_format(tf, secs, ampm))));
    }
    if let Some((d, f)) = parse_date_dmy(t, sys) {
        return Some((d, Some(f)));
    }
    for (i, _) in t.match_indices(' ') {
        let (a, b) = (t.get(..i)?, t.get(i + 1..)?);
        if let (Some((d, _)), Some((tf, _, _))) = (parse_date_dmy(a, sys), parse_time(b)) {
            return Some((d + tf, Some("m/d/yyyy h:mm")));
        }
    }
    None
}

/// `10:30 SA` / `2:30 CH` (sáng/chiều, vi-VN's AM/PM, as the formats show them) read as AM/PM.
fn vi_am_pm(t: &str) -> String {
    let trimmed = t.trim_end();
    let lower = trimmed.to_ascii_lowercase();
    for (vi, en) in [("sa", "AM"), ("ch", "PM")] {
        if let Some(head) = lower.strip_suffix(vi)
            && head.trim_end().ends_with(|c: char| c.is_ascii_digit())
            && let Some(original) = trimmed.get(..head.len())
        {
            return format!("{original}{en}");
        }
    }
    t.to_string()
}

fn parse_date_dmy(t: &str, sys: DateSystem) -> Option<(f64, &'static str)> {
    let t = t.trim().trim_end_matches(',');
    if t.contains('.') {
        return None;
    }
    fn num(s: &str) -> Option<i64> {
        let s = s.trim();
        if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) { s.parse::<i64>().ok() } else { None }
    }
    /// Two-digit years: 00–29 are 2000s, 30–99 the 1900s (as the en-US forms).
    fn expand(w: &str, y: i64) -> i64 {
        if w.trim().len() <= 2 { if y < 30 { 2000 + y } else { 1900 + y } } else { y }
    }
    let parts: Vec<&str> = t.split(['/', '-']).collect();
    if !parts.iter().all(|p| num(p).is_some()) {
        // Month names (`24-Oct-2026`, `Oct 24, 2026`) read as en-US reads them.
        return parse_date(t, sys);
    }
    match parts.as_slice() {
        [a, b, c] => {
            let (x, y, z) = (num(a)?, num(b)?, num(c)?);
            if a.trim().len() == 4 {
                return valid_ymd(sys, x, y, z).map(|s| (s, "yyyy-mm-dd"));
            }
            valid_ymd(sys, expand(c, z), y, x).map(|s| (s, "m/d/yyyy"))
        }
        [a, b] => {
            let (x, y) = (num(a)?, num(b)?);
            if (1..=31).contains(&x) && (1..=12).contains(&y) {
                return valid_ymd(sys, current_year(), y, x).map(|s| (s, "d-mmm"));
            }
            // m/yyyy
            if (1..=12).contains(&x) && b.trim().len() == 4 {
                return valid_ymd(sys, y, x, 1).map(|s| (s, "mmm-yy"));
            }
            None
        }
        _ => None,
    }
}

fn parse_datetime(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)> {
    if let Some((tf, secs, ampm)) = parse_time(t) {
        return Some((tf, Some(time_format(tf, secs, ampm))));
    }
    if let Some((d, f)) = parse_date(t, sys) {
        return Some((d, Some(f)));
    }
    // date + time: split at the last space(s) before a time-looking tail.
    for (i, _) in t.match_indices(' ') {
        let (a, b) = (t.get(..i)?, t.get(i + 1..)?);
        if let (Some((d, _)), Some((tf, _, _))) = (parse_date(a, sys), parse_time(b)) {
            return Some((d + tf, Some("m/d/yyyy h:mm")));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Parsed {
        parse_input(s, DateSystem::D1900)
    }

    #[test]
    fn month_name_short_year() {
        for s in ["Mar 4, 26", "Mar 4 26", "March 4 26", "4 March 26", "Mar 4, 2026"] {
            assert_eq!(p(s).value, Value::Number(46085.0), "{s}");
        }
        assert_eq!(p("Mar 4, 99").value, p("Mar 4, 1999").value);
        assert_eq!(p("Mar 4, 1776").value, Value::text("Mar 4, 1776"));
        assert_eq!(p("Feb 30, 26").value, Value::text("Feb 30, 26"));
    }

    #[test]
    fn numbers() {
        assert_eq!(p("42").value, Value::Number(42.0));
        assert_eq!(p("-3.5").value, Value::Number(-3.5));
        assert_eq!(p("1,234").format, Some("#,##0"));
        assert_eq!(p("1,234").value, Value::Number(1234.0));
        assert_eq!(p("1,23").value, Value::text("1,23"));
        assert_eq!(p("12%").value, Value::Number(0.12));
        assert_eq!(p("$1,200.50").value, Value::Number(1200.5));
        assert_eq!(p("$1,200.50").format, Some("\"$\"#,##0.00"));
        assert_eq!(p("(100)").value, Value::Number(-100.0));
        assert_eq!(p("1e3").value, Value::Number(1000.0));
        assert_eq!(p("1 1/2").value, Value::Number(1.5));
        assert_eq!(p(".5").value, Value::Number(0.5));
    }

    #[test]
    fn texts_and_specials() {
        assert_eq!(p("hello").value, Value::text("hello"));
        assert_eq!(p("'123").value, Value::text("123"));
        assert_eq!(p("true").value, Value::Bool(true));
        assert_eq!(p("#N/A").value, Value::Error(CellError::NA));
        assert_eq!(p("e").value, Value::text("e"));
        assert_eq!(p("-").value, Value::text("-"));
        assert_eq!(p("1.2.3.4").value, Value::text("1.2.3.4"));
    }

    #[test]
    fn dates_and_times() {
        assert_eq!(p("10/7/2026").value, Value::Number(46302.0));
        assert_eq!(p("2026-10-07").value, Value::Number(46302.0));
        assert_eq!(p("7-Oct-2026").value, Value::Number(46302.0));
        assert_eq!(p("Oct 7, 2026").value, Value::Number(46302.0));
        assert_eq!(p("2/30/2026").value, Value::text("2/30/2026"));
        assert_eq!(p("7/4/1776").value, Value::text("7/4/1776"));
        assert_eq!(p("1776-07-04").value, Value::text("1776-07-04"));
        assert_eq!(p("4-Jul-1776").value, Value::text("4-Jul-1776"));
        assert_eq!(p("12/31/1899").value, Value::text("12/31/1899"));
        assert_eq!(p("1/1/1900").value, Value::Number(1.0));
        assert_eq!(p("12:00").value, Value::Number(0.5));
        assert_eq!(p("6:00 PM").value, Value::Number(0.75));
        assert_eq!(p("12:00 AM").value, Value::Number(0.0));
        assert_eq!(p("10/7/2026 18:00").value, Value::Number(46302.75));
        assert_eq!(p("25:00").value, Value::Number(25.0 / 24.0));
        assert_eq!(p("25:00").format, Some("[h]:mm:ss"));
        assert_eq!(p("23:59").format, Some("h:mm"));
    }

    fn vi(s: &str) -> Parsed {
        parse_input_in(s, DateSystem::D1900, Locale::ViVn)
    }

    #[test]
    fn vietnamese_numbers() {
        // (typed, value, automatic format)
        let cases: &[(&str, f64, Option<&str>)] = &[
            ("1234,5", 1234.5, None),
            ("1.234,5", 1234.5, Some("#,##0.00")),
            ("1.234.567", 1_234_567.0, Some("#,##0")),
            ("-1,5", -1.5, None),
            ("(1.000)", -1000.0, Some("#,##0")),
            ("12,5%", 0.125, Some("0.00%")),
            ("1,5E+3", 1500.0, Some("0.00E+00")),
            ("1.234.567 ₫", 1_234_567.0, Some(VND_FORMAT)),
            ("1.234₫", 1234.0, Some(VND_FORMAT)),
            ("₫ 50", 50.0, Some(VND_FORMAT)),
            ("-1.234,5 ₫", -1234.5, Some(VND_FORMAT_DECIMALS)),
            ("1 1/2", 1.5, Some("# ?/?")),
        ];
        for (s, v, f) in cases {
            assert_eq!(vi(s).value, Value::Number(*v), "{s}");
            assert_eq!(vi(s).format, *f, "{s}");
        }
        // en-US spellings that mean something else here stay text, as Excel's vi-VN does.
        for s in ["1,234.5", "1.5", "1.23", "1,2,3", "₫", "1.234 ₫ ₫", "12%₫", "đồng", "Tiếng Việt", "1.2.3.4"] {
            assert_eq!(vi(s).value, Value::text(s), "{s}");
        }
        assert_eq!(vi("1.234").value, Value::Number(1234.0));
        assert_eq!(vi("'1,5").value, Value::text("1,5"));
        // A first group starting with 0 isn't grouping (phone numbers, en-US decimals).
        for s in ["0.125", "090.312.345", "-0.125", "0.125 ₫"] {
            assert_eq!(vi(s).value, Value::text(s), "{s}");
        }
        assert_eq!(vi("0,125").value, Value::Number(0.125));
        // The đồng in the ways people write it.
        for s in ["1.234đ", "1.234 Đ", "1.234 VNĐ", "1.234 vnd", "VND 1.234", "₫1.234"] {
            assert_eq!(vi(s).value, Value::Number(1234.0), "{s}");
            assert_eq!(vi(s).format, Some(VND_FORMAT), "{s}");
        }
        for s in ["đ", "VNĐ", "đồng", "1.234 đđ"] {
            assert_eq!(vi(s).value, Value::text(s), "{s}");
        }
        assert_eq!(vi("TRUE").value, Value::Bool(true));
        assert_eq!(vi("#N/A").value, Value::Error(CellError::NA));
        assert_eq!(parse_number_text_locale(" 1.234,5 ", DateSystem::D1900, Locale::ViVn), Some(1234.5));
        assert_eq!(parse_number_text_locale("1,234.5", DateSystem::D1900, Locale::ViVn), None);
        // en-US is unchanged.
        assert_eq!(parse_input_in("1,234.5", DateSystem::D1900, Locale::EnUs), p("1,234.5"));
    }

    /// Deeply nested parentheses never recurse: a 100,000-deep entry is text, not a stack overflow.
    #[test]
    fn nested_parentheses_are_bounded() {
        let deep = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
        for locale in Locale::ALL {
            assert_eq!(parse_input_in(&deep, DateSystem::D1900, locale).value, Value::text(deep.as_str()));
            assert_eq!(parse_input_in("((1))", DateSystem::D1900, locale).value, Value::text("((1))"));
            assert_eq!(parse_input_in("(1)", DateSystem::D1900, locale).value, Value::Number(-1.0));
        }
        assert_eq!(parse_number_text(&deep), None);
    }

    #[test]
    fn vietnamese_dates_and_times() {
        assert_eq!(vi("24/10/2026").value, p("10/24/2026").value);
        assert_eq!(vi("24/10/2026").format, Some("m/d/yyyy"));
        assert_eq!(vi("7/10/2026").value, Value::Number(46302.0));
        assert_eq!(vi("07-10-26").value, Value::Number(46302.0));
        assert_eq!(vi("2026-10-07").value, Value::Number(46302.0));
        assert_eq!(vi("7-Oct-2026").value, Value::Number(46302.0));
        assert_eq!(vi("31/2/2026").value, Value::text("31/2/2026"));
        assert_eq!(vi("13/13/2026").value, Value::text("13/13/2026"));
        assert_eq!(vi("24/10/2026 14:30").value, Value::Number(p("10/24/2026").value.as_f64().unwrap_or(0.0) + 14.5 / 24.0));
        assert_eq!(vi("24/10/2026 14:30").format, Some("m/d/yyyy h:mm"));
        assert_eq!(vi("14:30").value, Value::Number(14.5 / 24.0));
        assert_eq!(vi("14:30").format, Some("h:mm"));
        assert_eq!(vi("2:30 CH").value, Value::Number(14.5 / 24.0));
        assert_eq!(vi("2:30 CH").format, Some("h:mm AM/PM"));
        assert_eq!(vi("10:00 sa").value, Value::Number(10.0 / 24.0));
        assert_eq!(vi("sách").value, Value::text("sách"));
        assert_eq!(vi("24/10").format, Some("d-mmm"));
        assert_eq!(vi("10/2026").format, Some("mmm-yy"));
        // Hostile input never panics.
        for s in ["99999999999999999999/1/1", "1/1/99999999999999999999", "//", "-/-", "1/", "₫(1)", "((((1))))", "24/10/2026 99:99"] {
            let _ = vi(s);
        }
    }

    #[test]
    fn fictitious_1900_02_29() {
        // Excel accepts 29 February 1900 (serial 60) and keeps 1 March 1900 at 61.
        assert_eq!(p("2/29/1900").value, Value::Number(60.0));
        assert_eq!(p("1900-02-29").value, Value::Number(60.0));
        assert_eq!(p("29-Feb-1900").value, Value::Number(60.0));
        assert_eq!(p("2/28/1900").value, Value::Number(59.0));
        assert_eq!(p("3/1/1900").value, Value::Number(61.0));
        assert_eq!(p("2/29/1901").value, Value::text("2/29/1901"));
        assert_eq!(p("2/30/1900").value, Value::text("2/30/1900"));
        // The 1904 system starts after it.
        assert_eq!(parse_input("2/29/1900", DateSystem::D1904).value, Value::text("2/29/1900"));
        assert_eq!(parse_input("2/29/1904", DateSystem::D1904).value, Value::Number(59.0));
    }
}
