//! NexusOffice: format codes shown and edited in the regional format (Format Cells), stored en-US.

use gridcraft_numfmt::{Locale, delocalize_code, localize_code, localize_code_separators};

const CODES: &[&str] = &[
    "General",
    "0",
    "0.00",
    "#,##0",
    "#,##0.00",
    "#,##0;[Red]-#,##0",
    "0%",
    "0.00%",
    "0.00E+00",
    "# ?/?",
    "m/d/yyyy",
    "d-mmm-yy",
    "mmmm d, yyyy",
    "h:mm AM/PM",
    "h:mm:ss",
    "[h]:mm:ss",
    "hh:mm:ss.00",
    "dd.mm.yyyy",
    "@",
    "#,##0 [$₫-42A]",
    "#,##0.00 [$₫-42A];-#,##0.00 [$₫-42A]",
    "_(* #,##0.00_);_(* (#,##0.00);_(* \"-\"??_);_(@_)",
    "[<1000.5]0.0;#,##0",
    "\"1,000.5\" 0.0",
    "\\. 0.00",
    "#,##0,",
    "[$-F800]dddd, mmmm d, yyyy",
    "",
    "[",
    "\"",
    ";;;",
];

#[test]
fn codes_show_in_vietnamese() {
    use Locale::ViVn;
    for (en, vi) in [
        ("#,##0.00", "#.##0,00"),
        ("0.0%", "0,0%"),
        ("#,##0 [$₫-42A]", "#.##0 [$₫-42A]"),
        ("\"1,000.5\" 0.0", "\"1,000.5\" 0,0"),
        ("hh:mm:ss.00", "hh:mm:ss,00"),
        ("dd.mm.yyyy", "dd.mm.yyyy"),
        ("mmmm d, yyyy", "mmmm d, yyyy"),
        ("[<1000.5]0.0;#,##0", "[<1000,5]0,0;#.##0"),
        ("[Red]#,##0", "[Red]#.##0"),
        ("m/d/yyyy", "dd/mm/yyyy"),
        ("m/d/yyyy h:mm", "dd/mm/yyyy h:mm"),
        ("General", "General"),
    ] {
        assert_eq!(localize_code(en, ViVn), vi, "{en}");
    }
    // The date order applies to the system short date only; other date codes keep their shape.
    assert_eq!(localize_code("d-mmm-yy", ViVn), "d-mmm-yy");
    assert_eq!(localize_code("mm/dd/yyyy", ViVn), "mm/dd/yyyy");
}

#[test]
fn separators_round_trip_and_en_us_is_unchanged() {
    for c in CODES {
        let vi = localize_code_separators(c, Locale::ViVn);
        assert_eq!(delocalize_code(&vi, Locale::ViVn), *c, "{c} → {vi}");
        assert_eq!(localize_code(c, Locale::EnUs), *c);
        assert_eq!(delocalize_code(c, Locale::EnUs), *c);
    }
}

#[test]
fn hostile_codes_never_panic() {
    let long = "#,##0.00;".repeat(5000);
    for c in ["[<", "[<1,5", "[<1.5", "\\", "_", "*", "\"\"\"", "ss.", "s.0", "[h]:mm:ss.000", "e\u{302}\u{323}", "\u{0}", long.as_str()] {
        for l in Locale::ALL {
            let a = localize_code(c, l);
            let _ = delocalize_code(&a, l);
            let _ = delocalize_code(c, l);
        }
    }
}
