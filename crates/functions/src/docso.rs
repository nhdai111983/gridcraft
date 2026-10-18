//! DOCSO: a whole amount of money spelled out in Vietnamese words.
//!
//! Not an Excel function; it is the number-to-words helper Vietnamese accountants add to their
//! workbooks for receipts and payment orders. Kept apart from `text.rs` so syncing upstream
//! stays free of conflicts.

use gridcraft_core::{CellError, Value};

use crate::util::{R, has, num, round_half_away, text, text_val};
use crate::{Arg, Ctx};

/// Amounts of 10^15 and beyond have no reading here (the largest scale word is "nghìn tỷ").
const LIMIT: f64 = 1e15;
const DIGITS: [&str; 10] = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];
/// Scale word of each group of three digits, least significant group first.
const SCALES: [&str; 3] = ["", "nghìn", "triệu"];
const BILLION: u64 = 1_000_000_000;

fn digit(d: u64) -> &'static str {
    usize::try_from(d).ok().and_then(|i| DIGITS.get(i)).copied().unwrap_or_default()
}

/// Words for one group of three digits (`0..=999`). `padded` is true when a higher group has
/// already been read, which makes an empty hundreds place "không trăm" and a bare unit after an
/// empty tens place "linh X" ("một triệu không trăm linh năm").
fn group_words(g: u64, padded: bool) -> String {
    let (h, t, u) = (g / 100 % 10, g / 10 % 10, g % 10);
    let mut w: Vec<&str> = Vec::new();
    let hundreds = h > 0 || padded;
    if hundreds {
        w.push(digit(h));
        w.push("trăm");
    }
    match t {
        0 if u > 0 && hundreds => w.push("linh"),
        0 => {}
        1 => w.push("mười"),
        _ => {
            w.push(digit(t));
            w.push("mươi");
        }
    }
    match (u, t) {
        (0, _) => {}
        (1, 2..) => w.push("mốt"),
        (4, 2..) => w.push("tư"),
        (5, 1..) => w.push("lăm"),
        _ => w.push(digit(u)),
    }
    w.join(" ")
}

/// Words for `n < 10^9` as groups of three digits (triệu, nghìn, units). `padded` carries over
/// from a higher part already read.
fn below_billion(n: u64, padded: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (i, scale) in SCALES.iter().enumerate().rev() {
        let g = n / 1000u64.pow(i as u32) % 1000;
        if g == 0 {
            continue;
        }
        let mut s = group_words(g, padded || !parts.is_empty());
        if !scale.is_empty() {
            s.push(' ');
            s.push_str(scale);
        }
        parts.push(s);
    }
    parts.join(" ")
}

/// Vietnamese words for a whole number (lower case, no sign, no unit). Works on groups of three
/// digits taken with `%` and `/`, so nothing multiplies or overflows. Billions are counted as a
/// number of "tỷ" of their own: 10^12 is "một nghìn tỷ" and 1.5 × 10^12 "một nghìn năm trăm tỷ".
pub(crate) fn vi_number_words(n: u64) -> String {
    if n == 0 {
        return digit(0).to_string();
    }
    let (billions, rest) = (n / BILLION, n % BILLION);
    let mut out = String::new();
    if billions > 0 {
        // The tỷ count is read as a number of its own (recursion ends: it is far smaller than `n`).
        out.push_str(&vi_number_words(billions));
        out.push_str(" tỷ");
    }
    if rest > 0 {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&below_billion(rest, billions > 0));
    }
    out
}

/// DOCSO(number, [unit]): the amount in Vietnamese words, rounded to a whole number. It starts
/// with a capital letter, a negative amount with "Âm", and unless `unit` is empty the words are
/// followed by the unit (default "đồng") and "chẵn".
pub(crate) fn docso(a: &[Arg], _c: &mut dyn Ctx) -> R<Value> {
    let x = round_half_away(num(a, 0)?, 0);
    if !x.is_finite() || x.abs() >= LIMIT {
        return Err(CellError::Num);
    }
    let unit = if has(a, 1) { text(a, 1)? } else { "đồng".to_string() };
    // `x` is a whole number below 10^15 here, so the cast is exact.
    let n = x.abs() as u64;
    let mut words = vi_number_words(n);
    if x < 0.0 && n > 0 {
        words = format!("âm {words}");
    }
    let mut out = String::with_capacity(words.len() + unit.len() + 8);
    let mut chars = words.chars();
    if let Some(first) = chars.next() {
        out.extend(first.to_uppercase());
        out.push_str(chars.as_str());
    }
    if !unit.is_empty() {
        out.push(' ');
        out.push_str(&unit);
        out.push_str(" chẵn");
    }
    text_val(out)
}

#[cfg(test)]
mod tests {
    use gridcraft_core::{CellError, Value};

    use crate::util::testutil::*;

    #[test]
    fn docso_reads_vietnamese_amounts() {
        let cases: &[(f64, &str)] = &[
            (0.0, "Không đồng chẵn"),
            (1.0, "Một đồng chẵn"),
            (5.0, "Năm đồng chẵn"),
            (10.0, "Mười đồng chẵn"),
            (11.0, "Mười một đồng chẵn"),
            (14.0, "Mười bốn đồng chẵn"),
            (15.0, "Mười lăm đồng chẵn"),
            (20.0, "Hai mươi đồng chẵn"),
            (21.0, "Hai mươi mốt đồng chẵn"),
            (24.0, "Hai mươi tư đồng chẵn"),
            (25.0, "Hai mươi lăm đồng chẵn"),
            (34.0, "Ba mươi tư đồng chẵn"),
            (91.0, "Chín mươi mốt đồng chẵn"),
            (99.0, "Chín mươi chín đồng chẵn"),
            (100.0, "Một trăm đồng chẵn"),
            (101.0, "Một trăm linh một đồng chẵn"),
            (105.0, "Một trăm linh năm đồng chẵn"),
            (110.0, "Một trăm mười đồng chẵn"),
            (115.0, "Một trăm mười lăm đồng chẵn"),
            (121.0, "Một trăm hai mươi mốt đồng chẵn"),
            (404.0, "Bốn trăm linh bốn đồng chẵn"),
            (999.0, "Chín trăm chín mươi chín đồng chẵn"),
            (1_000.0, "Một nghìn đồng chẵn"),
            (1_001.0, "Một nghìn không trăm linh một đồng chẵn"),
            (1_010.0, "Một nghìn không trăm mười đồng chẵn"),
            (1_234.0, "Một nghìn hai trăm ba mươi tư đồng chẵn"),
            (10_500.0, "Mười nghìn năm trăm đồng chẵn"),
            (21_000.0, "Hai mươi mốt nghìn đồng chẵn"),
            (100_000.0, "Một trăm nghìn đồng chẵn"),
            (100_005.0, "Một trăm nghìn không trăm linh năm đồng chẵn"),
            (999_999.0, "Chín trăm chín mươi chín nghìn chín trăm chín mươi chín đồng chẵn"),
            (1_000_000.0, "Một triệu đồng chẵn"),
            (1_000_005.0, "Một triệu không trăm linh năm đồng chẵn"),
            (1_000_500.0, "Một triệu năm trăm đồng chẵn"),
            (1_000_050.0, "Một triệu không trăm năm mươi đồng chẵn"),
            (1_250_000.0, "Một triệu hai trăm năm mươi nghìn đồng chẵn"),
            (5_000_000.0, "Năm triệu đồng chẵn"),
            (12_345_678.0, "Mười hai triệu ba trăm bốn mươi lăm nghìn sáu trăm bảy mươi tám đồng chẵn"),
            (1_000_000_000.0, "Một tỷ đồng chẵn"),
            (2_500_000_000.0, "Hai tỷ năm trăm triệu đồng chẵn"),
            (1_000_000_001.0, "Một tỷ không trăm linh một đồng chẵn"),
            (1_001_000_000.0, "Một tỷ không trăm linh một triệu đồng chẵn"),
            (1_000_000_000_000.0, "Một nghìn tỷ đồng chẵn"),
            (1_500_000_000_000.0, "Một nghìn năm trăm tỷ đồng chẵn"),
            (
                999_999_999_999_999.0,
                "Chín trăm chín mươi chín nghìn chín trăm chín mươi chín tỷ chín trăm chín mươi chín triệu chín trăm chín mươi chín nghìn chín trăm chín mươi chín đồng chẵn",
            ),
            (-5.0, "Âm năm đồng chẵn"),
            (-1_234.0, "Âm một nghìn hai trăm ba mươi tư đồng chẵn"),
            (-21.0, "Âm hai mươi mốt đồng chẵn"),
            // Rounded half away from zero to a whole amount.
            (1.4, "Một đồng chẵn"),
            (1.5, "Hai đồng chẵn"),
            (2.5, "Ba đồng chẵn"),
            (-2.5, "Âm ba đồng chẵn"),
            (0.4, "Không đồng chẵn"),
            (-0.4, "Không đồng chẵn"),
            (
                999_999_999_999_999.4,
                "Chín trăm chín mươi chín nghìn chín trăm chín mươi chín tỷ chín trăm chín mươi chín triệu chín trăm chín mươi chín nghìn chín trăm chín mươi chín đồng chẵn",
            ),
        ];
        assert!(cases.len() >= 50);
        for (x, want) in cases {
            is_text(ev("DOCSO", vec![n(*x)]), want);
        }
        // Numeric text and booleans coerce like any number argument.
        is_text(ev("DOCSO", vec![t("1250000")]), "Một triệu hai trăm năm mươi nghìn đồng chẵn");
        // A custom unit replaces "đồng"; "chẵn" still follows.
        is_text(ev("DOCSO", vec![n(5_000_000.0), t("VND")]), "Năm triệu VND chẵn");
        is_text(ev("DOCSO", vec![n(21.0), t("đô la")]), "Hai mươi mốt đô la chẵn");
    }

    #[test]
    fn vi_number_words_is_total() {
        use super::vi_number_words;
        assert_eq!(vi_number_words(0), "không");
        assert_eq!(vi_number_words(1_000_000_000_000), "một nghìn tỷ");
        // Far beyond what DOCSO accepts, but the helper never panics or loses digits.
        assert!(!vi_number_words(u64::MAX).is_empty());
    }

    #[test]
    fn docso_errors() {
        is_err(ev("DOCSO", vec![t("abc")]), CellError::Value);
        is_err(ev("DOCSO", vec![n(1e15)]), CellError::Num);
        is_err(ev("DOCSO", vec![n(-1e15)]), CellError::Num);
        is_err(ev("DOCSO", vec![n(f64::MAX)]), CellError::Num);
        is_err(ev("DOCSO", vec![n(f64::MIN)]), CellError::Num);
        is_err(ev("DOCSO", vec![n(1.0), t("đồng"), t("x")]), CellError::Value);
        is_err(ev("DOCSO", vec![]), CellError::Value);
        is_err(ev("DOCSO", vec![e(CellError::NA)]), CellError::NA);
        // Just below the limit still reads.
        assert!(matches!(ev("DOCSO", vec![n(999_999_999_999_999.0)]), Value::Text(_)));
    }

    #[test]
    fn docso_empty_unit_has_no_chan() {
        is_text(ev("DOCSO", vec![n(5_000_000.0), t("")]), "Năm triệu");
        is_text(ev("DOCSO", vec![n(-105.0), t("")]), "Âm một trăm linh năm");
        // Omitting the unit entirely is not the same as an empty unit.
        is_text(ev("DOCSO", vec![n(5_000_000.0)]), "Năm triệu đồng chẵn");
    }
}
