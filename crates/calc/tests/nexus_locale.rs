//! NexusOffice: formulas coerce in the workbook's regional format, as Excel's coerce in the
//! system's. In a vi-VN workbook the criterion `">1,5"` means 1.5, `="1,5"+1` is 2.5 and
//! `=1.5&""` is `"1,5"`; an en-US workbook is unchanged. Formulas are stored en-US either way.

use gridcraft_calc::Calc;
use gridcraft_core::date::serial_from_ymd;
use gridcraft_core::{CellError, CellRef, DateSystem, Locale, Value, coercion_locale};
use gridcraft_model::{Cell, Formula, Workbook};

fn at(a1: &str) -> CellRef {
    CellRef::parse(a1).unwrap()
}

struct Book {
    wb: Workbook,
    calc: Calc,
}

impl Book {
    fn new(locale: Locale) -> Book {
        let mut wb = Workbook::new();
        wb.locale = locale;
        Book { wb, calc: Calc::new() }
    }
    fn put(&mut self, a1: &str, cell: Cell) {
        self.wb.sheet_mut(0).unwrap().cells.set(at(a1), cell);
        self.calc.cells_changed(&mut self.wb, &[(0, at(a1))]);
    }
    fn num(&mut self, a1: &str, n: f64) {
        self.put(a1, Cell::value(Value::Number(n)));
    }
    fn text(&mut self, a1: &str, t: &str) {
        self.put(a1, Cell::value(Value::text(t)));
    }
    /// A formula in its stored (en-US) syntax; string literals are kept as written.
    fn formula(&mut self, a1: &str, f: &str) -> Value {
        self.put(a1, Cell::formula(Formula::new(f)));
        self.get(a1)
    }
    fn get(&self, a1: &str) -> Value {
        self.wb.sheet(0).unwrap().value(at(a1))
    }
}

/// A1:A4 = 1, 1.5, 2, 2.5; B1:B3 = 24/10/2026, 25/10/2026, 26/10/2026; C1:C4 = 10, 20, 30, 40.
fn data(locale: Locale) -> Book {
    let mut b = Book::new(locale);
    for (i, n) in [1.0, 1.5, 2.0, 2.5].into_iter().enumerate() {
        b.num(&format!("A{}", i + 1), n);
        b.num(&format!("C{}", i + 1), 10.0 * (i + 1) as f64);
    }
    for (i, d) in [24, 25, 26].into_iter().enumerate() {
        b.num(&format!("B{}", i + 1), serial_from_ymd(DateSystem::D1900, 2026, 10, d).unwrap());
    }
    b
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

#[test]
fn criteria_read_the_workbooks_regional_format() {
    let mut vi = data(Locale::ViVn);
    // Decimal comma in comparison criteria (COUNTIF, SUMIF, AVERAGEIF and the *IFS family).
    assert_eq!(vi.formula("E1", "=COUNTIF(A1:A4,\">1,5\")"), n(2.0));
    assert_eq!(vi.formula("E2", "=SUMIF(A1:A4,\">=1,5\",C1:C4)"), n(90.0));
    assert_eq!(vi.formula("E3", "=AVERAGEIF(A1:A4,\"<2,5\",C1:C4)"), n(20.0));
    assert_eq!(vi.formula("E4", "=COUNTIFS(A1:A4,\">1\",A1:A4,\"<=2,5\")"), n(3.0));
    assert_eq!(vi.formula("E5", "=SUMIFS(C1:C4,A1:A4,\"1,5\")"), n(20.0));
    assert_eq!(vi.formula("E6", "=AVERAGEIFS(C1:C4,A1:A4,\">=2\")"), n(35.0));
    assert_eq!(vi.formula("E7", "=MAXIFS(C1:C4,A1:A4,\"<>2,5\")"), n(30.0));
    assert_eq!(vi.formula("E8", "=MINIFS(C1:C4,A1:A4,\">1,2\")"), n(20.0));
    // Dates day first.
    assert_eq!(vi.formula("F1", "=COUNTIF(B1:B3,\"25/10/2026\")"), n(1.0));
    assert_eq!(vi.formula("F2", "=COUNTIF(B1:B3,\">=25/10/2026\")"), n(2.0));
    // An en-US spelling is text in vi-VN, as Excel reads it with vi-VN settings: no number matches.
    assert_eq!(vi.formula("F3", "=COUNTIF(A1:A4,\">1.5\")"), n(0.0));
    // A number joined into a criterion is spelled the vi-VN way, so `">"&cell` keeps working.
    assert_eq!(vi.formula("F4", "=COUNTIF(A1:A4,\">\"&A2)"), n(2.0));
    assert_eq!(vi.formula("F5", "=COUNTIF(B1:B3,\"<\"&B2)"), n(1.0));
    // Numeric text in the range equals a number criterion when it reads as that number.
    vi.text("A5", "1,5");
    assert_eq!(vi.formula("F6", "=COUNTIF(A1:A5,1.5)"), n(2.0));
    // Database functions: the criteria cell is a criterion too.
    vi.text("H1", "Qty");
    vi.text("H2", ">1,5");
    vi.text("I1", "Qty");
    for (i, x) in [1.0, 1.5, 2.0, 2.5].into_iter().enumerate() {
        vi.num(&format!("I{}", i + 2), x);
    }
    assert_eq!(vi.formula("F7", "=DCOUNT(I1:I5,\"Qty\",H1:H2)"), n(2.0));
}

#[test]
fn criteria_in_en_us_are_unchanged() {
    let mut en = data(Locale::EnUs);
    assert_eq!(en.formula("E1", "=COUNTIF(A1:A4,\">1.5\")"), n(2.0));
    assert_eq!(en.formula("E2", "=SUMIF(A1:A4,\">=1.5\",C1:C4)"), n(90.0));
    assert_eq!(en.formula("E3", "=COUNTIF(A1:A4,\">\"&A2)"), n(2.0));
    assert_eq!(en.formula("E4", "=COUNTIF(B1:B3,\"10/25/2026\")"), n(1.0));
    assert_eq!(en.formula("E5", "=COUNTIFS(A1:A4,\">1\",A1:A4,\"<=2.5\")"), n(3.0));
    // en-US reads `1,5` with a thousands separator, as before.
    assert_eq!(en.formula("E6", "=COUNTIF(A1:A4,\"1,5\")"), Value::Number(0.0));
}

#[test]
fn arithmetic_coerces_text_in_the_workbooks_regional_format() {
    let mut vi = Book::new(Locale::ViVn);
    assert_eq!(vi.formula("A1", "=\"1,5\"+1"), n(2.5));
    assert_eq!(vi.formula("A2", "=\"1.234,5\"*2"), n(2469.0));
    assert_eq!(vi.formula("A3", "=-\"2,5\""), n(-2.5));
    assert_eq!(vi.formula("A4", "=SQRT(\"2,25\")"), n(1.5));
    assert_eq!(vi.formula("A5", "=\"25/10/2026\"+0"), n(serial_from_ymd(DateSystem::D1900, 2026, 10, 25).unwrap()));
    assert_eq!(vi.formula("A6", "=\"1.5\"+1"), Value::Error(CellError::Value));
    assert_eq!(vi.formula("A11", "=DATEVALUE(\"25/10/2026\")"), n(serial_from_ymd(DateSystem::D1900, 2026, 10, 25).unwrap()));
    assert_eq!(vi.formula("A12", "=TIMEVALUE(\"14:30\")"), n(14.5 / 24.0));
    // A cell holding the text `1,5` (say, pasted with an apostrophe) coerces the same way.
    vi.text("B1", "1,5");
    assert_eq!(vi.formula("A7", "=B1*2"), n(3.0));
    // Number → text spells the decimal comma: joining, text functions.
    assert_eq!(vi.formula("A8", "=1.5&\"\""), Value::text("1,5"));
    assert_eq!(vi.formula("A9", "=LEFT(1234.5,6)"), Value::text("1234,5"));
    assert_eq!(vi.formula("A10", "=(1.5&\"\")+1"), n(2.5));
    // VALUETOTEXT / ARRAYTOTEXT spell numbers as `&` does, with the formula syntax's separators.
    assert_eq!(vi.formula("A13", "=VALUETOTEXT(1.5)"), Value::text("1,5"));
    assert_eq!(vi.formula("A14", "=ARRAYTOTEXT({1.5,2;3,4})"), Value::text("1,5; 2; 3; 4"));
    assert_eq!(vi.formula("A15", "=ARRAYTOTEXT({1.5,2;3,4},1)"), Value::text("{1,5\\2;3\\4}"));
    // A database field named by a number matches the number label spelled the vi-VN way.
    vi.num("D1", 1.5);
    vi.num("D2", 7.0);
    vi.num("E1", 1.5);
    vi.num("E2", 7.0);
    assert_eq!(vi.formula("A16", "=DSUM(D1:D2,\"1,5\",E1:E2)"), n(7.0));
    // Outside an evaluation everything is en-US again.
    assert_eq!(coercion_locale(), Locale::EnUs);
    assert_eq!(Value::Number(1.5).to_text(), Ok("1.5".to_string()));

    let mut en = Book::new(Locale::EnUs);
    assert_eq!(en.formula("A1", "=\"1.5\"+1"), n(2.5));
    assert_eq!(en.formula("A2", "=1.5&\"\""), Value::text("1.5"));
    assert_eq!(en.formula("A3", "=\"10/25/2026\"+0"), n(serial_from_ymd(DateSystem::D1900, 2026, 10, 25).unwrap()));
    assert_eq!(en.formula("A4", "=\"25/10/2026\"+0"), Value::Error(CellError::Value));
    assert_eq!(en.formula("A7", "=ARRAYTOTEXT({1.5,2;3,4},1)"), Value::text("{1.5,2;3,4}"));
    assert_eq!(en.formula("A8", "=ARRAYTOTEXT({1.5,2;3,4})"), Value::text("1.5, 2, 3, 4"));
    assert_eq!(en.formula("A5", "=DATEVALUE(\"10/25/2026\")"), n(serial_from_ymd(DateSystem::D1900, 2026, 10, 25).unwrap()));
    assert_eq!(en.formula("A6", "=DATEVALUE(\"25/10/2026\")"), Value::Error(CellError::Value));
}

#[test]
fn hostile_criteria_and_coercions_never_panic() {
    let junk = [
        "",
        ">",
        "<",
        "=",
        "<>",
        ">=",
        "<=",
        ">,",
        ">=,",
        "1,5,5",
        "1.2.3,4",
        ",",
        ".",
        ",,",
        "..",
        "₫",
        "đ",
        "(",
        "((1))",
        "(1,5)",
        "1,5₫",
        "1e999",
        "-1e999",
        "1,5E+400",
        "99/99/9999",
        "31/02/2026",
        "0/0/0",
        "1/1/-1",
        "25/10/2026 25:61",
        "12:30 CH",
        "SA",
        "1.234.567.890.123.456.789,5",
        "%",
        "1,5%%",
        "\u{0}",
        "~",
        "*",
        "?",
        "ẮẰẲẴẶ",
        "e\u{302}\u{323}",
        "\u{fffd}",
    ];
    for loc in Locale::ALL {
        let mut b = data(loc);
        for (i, j) in junk.iter().enumerate() {
            let lit = j.replace('"', "\"\"");
            let row = i + 1;
            for (col, f) in [
                ("K", format!("=COUNTIF(A1:A4,\"{lit}\")")),
                ("L", format!("=SUMIFS(C1:C4,A1:A4,\"{lit}\",B1:B4,\"<>{lit}\")")),
                ("M", format!("=\"{lit}\"+1")),
                ("N", format!("=\"{lit}\"&1.5")),
            ] {
                let _ = b.formula(&format!("{col}{row}"), &f);
            }
        }
        let long = "1,5".repeat(2000);
        let _ = b.formula("O1", &format!("=COUNTIF(A1:A4,\">{long}\")"));
        let _ = b.formula("O2", &format!("=\"{long}\"+1"));
    }
    assert_eq!(coercion_locale(), Locale::EnUs);
}
