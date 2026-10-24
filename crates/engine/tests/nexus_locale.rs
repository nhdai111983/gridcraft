//! NexusOffice: the vi-VN regional format. Values typed `1.234,5`, `24/10/2026` or `1.234 ₫`
//! (`cell.set`'s `locale`) and shown that way ([`Session::set_locale`]), while files, formulas and
//! the API stay en-US. CSV follows the regional format both ways, as Excel's follows the system's.

use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Locale, Value};
use serde_json::json;

fn at(a1: &str) -> CellRef {
    CellRef::parse(a1).unwrap()
}

fn value(s: &Session, a1: &str) -> Value {
    s.active().unwrap().wb.active().unwrap().value(at(a1))
}

fn text(s: &Session, a1: &str) -> String {
    let d = s.active().unwrap();
    gridcraft_engine::display::cell_text(&d.wb, d.wb.active().unwrap(), at(a1))
}

fn code(s: &Session, a1: &str) -> String {
    let d = s.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(at(a1))).num_fmt.as_str().to_string()
}

fn set(s: &mut Session, a1: &str, input: &str, locale: Option<&str>) {
    let mut p = json!({"cell": a1, "input": input});
    if let Some(l) = locale {
        p["locale"] = json!(l);
    }
    s.execute("cell.set", p).unwrap();
}

fn session() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

/// The standard test text, precomposed (NFC) and decomposed (NFD).
const VI_NFC: &str = "Tiếng Việt: ẮẰẲẴẶ ắằẳẵặ ẤẦẨẪẬ ấầẩẫậ Đđ ỆệỞởỮữỰựỸỹ — Nguyễn Thị Ngọc Ánh, Hà Nội, Đắk Lắk";

fn nfd(s: &str) -> String {
    // Enough of NFD for the test text: split the letters it uses into base + combining marks.
    let table: &[(char, &str)] = &[
        ('ế', "e\u{302}\u{301}"),
        ('ệ', "e\u{302}\u{323}"),
        ('Ệ', "E\u{302}\u{323}"),
        ('ễ', "e\u{302}\u{303}"),
        ('ị', "i\u{323}"),
        ('ọ', "o\u{323}"),
        ('Á', "A\u{301}"),
        ('à', "a\u{300}"),
        ('ộ', "o\u{302}\u{323}"),
        ('ắ', "a\u{306}\u{301}"),
        ('Ắ', "A\u{306}\u{301}"),
        ('ằ', "a\u{306}\u{300}"),
        ('Ằ', "A\u{306}\u{300}"),
        ('ẳ', "a\u{306}\u{309}"),
        ('Ẳ', "A\u{306}\u{309}"),
        ('ẵ', "a\u{306}\u{303}"),
        ('Ẵ', "A\u{306}\u{303}"),
        ('ặ', "a\u{306}\u{323}"),
        ('Ặ', "A\u{306}\u{323}"),
        ('ấ', "a\u{302}\u{301}"),
        ('Ấ', "A\u{302}\u{301}"),
        ('ầ', "a\u{302}\u{300}"),
        ('Ầ', "A\u{302}\u{300}"),
        ('ẩ', "a\u{302}\u{309}"),
        ('Ẩ', "A\u{302}\u{309}"),
        ('ẫ', "a\u{302}\u{303}"),
        ('Ẫ', "A\u{302}\u{303}"),
        ('ậ', "a\u{302}\u{323}"),
        ('Ậ', "A\u{302}\u{323}"),
        ('ở', "o\u{31b}\u{309}"),
        ('Ở', "O\u{31b}\u{309}"),
        ('ữ', "u\u{31b}\u{303}"),
        ('Ữ', "U\u{31b}\u{303}"),
        ('ự', "u\u{31b}\u{323}"),
        ('Ự', "U\u{31b}\u{323}"),
        ('ỹ', "y\u{303}"),
        ('Ỹ', "Y\u{303}"),
        ('ễ', "e\u{302}\u{303}"),
    ];
    s.chars().map(|c| table.iter().find(|(k, _)| *k == c).map_or_else(|| c.to_string(), |(_, v)| (*v).to_string())).collect()
}

#[test]
fn values_typed_in_vietnamese() {
    let mut s = session();
    // (typed, value, stored format)
    let cases: &[(&str, Value, &str)] = &[
        ("1.234,5", Value::Number(1234.5), "#,##0.00"),
        ("1234,5", Value::Number(1234.5), "General"),
        ("-0,25", Value::Number(-0.25), "General"),
        ("1.234.567 ₫", Value::Number(1_234_567.0), "#,##0 [$₫-42A]"),
        ("24/10/2026", Value::Number(46319.0), "m/d/yyyy"),
        ("14:30", Value::Number(14.5 / 24.0), "h:mm"),
        ("12,5%", Value::Number(0.125), "0.00%"),
        ("1,234.5", Value::text("1,234.5"), "General"),
    ];
    for (i, (typed, v, fmt)) in cases.iter().enumerate() {
        let a1 = format!("A{}", i + 1);
        set(&mut s, &a1, typed, Some("vi-VN"));
        assert_eq!(value(&s, &a1), *v, "{typed}");
        assert_eq!(code(&s, &a1), *fmt, "{typed}");
    }
    // Text is kept as typed, NFC or NFD.
    let decomposed = nfd(VI_NFC);
    assert_ne!(decomposed, VI_NFC);
    set(&mut s, "B1", VI_NFC, Some("vi"));
    set(&mut s, "B2", &decomposed, Some("vi"));
    assert_eq!(value(&s, "B1"), Value::text(VI_NFC));
    assert_eq!(value(&s, "B2"), Value::text(&decomposed));
    // Formulas are en-US whatever the locale (the interface translates its syntax first).
    set(&mut s, "C1", "=SUM(1.5,2)", Some("vi-VN"));
    assert_eq!(value(&s, "C1"), Value::Number(3.5));
    // Without `locale` the API reads en-US, and an unknown locale is refused.
    set(&mut s, "D1", "1.234,5", None);
    assert_eq!(value(&s, "D1"), Value::text("1.234,5"));
    assert!(s.execute("cell.set", json!({"cell": "D2", "input": "1", "locale": "xx"})).is_err());
    s.execute("selection.set", json!({"range": "E1:E3"})).unwrap();
    s.execute("range.fill", json!({"input": "2,5", "locale": "vi-VN"})).unwrap();
    assert_eq!(value(&s, "E3"), Value::Number(2.5));
}

#[test]
fn display_follows_the_session_locale_and_the_file_stays_en_us() {
    let mut s = session();
    set(&mut s, "A1", "1234567.891", None);
    s.execute("home.numberFormat", json!({"range": "A1", "code": "#,##0.00"})).unwrap();
    set(&mut s, "A2", "10/24/2026", None);
    set(&mut s, "A3", "=TEXT(1234.5,\"#.##0,00\")", None);
    set(&mut s, "A4", "=VALUE(\"1.234,5\")", None);
    set(&mut s, "A5", "1234.5", None);
    s.execute("home.numberFormat", json!({"range": "A6", "code": "#,##0 [$₫-42A]"})).unwrap();
    set(&mut s, "A6", "1234567", None);
    assert_eq!(text(&s, "A1"), "1,234,567.89");
    assert_eq!(text(&s, "A2"), "10/24/2026");
    assert_eq!(value(&s, "A4"), Value::Error(gridcraft_engine::core::CellError::Value));
    let en_text = value(&s, "A3");
    // Switching shows the same values the vi-VN way, recalculates TEXT/VALUE, keeps it saved.
    s.active_mut().unwrap().saved = s.active().unwrap().wb.clone();
    s.set_locale(Locale::ViVn);
    assert!(!s.active().unwrap().is_dirty(), "the locale is not part of the file");
    assert_eq!(text(&s, "A1"), "1.234.567,89");
    assert_eq!(text(&s, "A2"), "24/10/2026");
    assert_eq!(value(&s, "A3"), Value::text("1.234,50"));
    assert_ne!(value(&s, "A3"), en_text);
    assert_eq!(value(&s, "A4"), Value::Number(1234.5));
    assert_eq!(text(&s, "A5"), "1234,5");
    assert_eq!(text(&s, "A6"), "1.234.567 ₫");
    // The formula bar's text reads back in the same locale.
    let d = s.active().unwrap();
    assert_eq!(gridcraft_engine::display::edit_text(&d.wb, d.wb.active().unwrap(), at("A5")), "1234,5");
    assert_eq!(gridcraft_engine::display::edit_text(&d.wb, d.wb.active().unwrap(), at("A2")), "24/10/2026");
    // XLSX stays locale-neutral: en-US numbers and codes, reopened identically.
    let saved = s.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let bytes = gridcraft_engine::io::base64_decode(saved["base64"].as_str().unwrap()).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes.clone())).unwrap();
    let mut sheet = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("xl/worksheets/sheet1.xml").unwrap(), &mut sheet).unwrap();
    assert!(sheet.contains("<v>1234567.891</v>") && sheet.contains("<v>1234.5</v>"), "{sheet}");
    assert!(!sheet.contains("1234567,891"));
    let mut styles = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("xl/styles.xml").unwrap(), &mut styles).unwrap();
    assert!(styles.contains("formatCode=\"#,##0.00\""), "{styles}");
    let (back, _) = gridcraft_engine::io::open_bytes("book.xlsx", &bytes).unwrap();
    assert_eq!(back.locale, Locale::EnUs);
    assert_eq!(back.sheet(0).unwrap().value(at("A1")), Value::Number(1234567.891));
    assert_eq!(back.sheet(0).unwrap().value(at("A2")), Value::Number(46319.0));
    // Back to en-US.
    s.set_locale(Locale::EnUs);
    assert_eq!(text(&s, "A1"), "1,234,567.89");
    assert_eq!(value(&s, "A3"), en_text);
}

/// CSV in vi-VN: `;` between fields and the displayed values, decimal comma included (Excel
/// uses the system's list separator); reading it back in vi-VN gives the same values.
#[test]
fn csv_round_trips_in_vietnamese() {
    let mut s = session();
    s.set_locale(Locale::ViVn);
    set(&mut s, "A1", "1.234,5", Some("vi-VN"));
    set(&mut s, "B1", "24/10/2026", Some("vi-VN"));
    set(&mut s, "C1", "1.234.567 ₫", Some("vi-VN"));
    set(&mut s, "D1", "0,000123456789", Some("vi-VN"));
    set(&mut s, "E1", "a;b", Some("vi-VN"));
    set(&mut s, "F1", VI_NFC, Some("vi-VN"));
    let saved = s.execute("file.saveBytes", json!({"format": "csv"})).unwrap();
    let bytes = gridcraft_engine::io::base64_decode(saved["base64"].as_str().unwrap()).unwrap();
    let csv = String::from_utf8(bytes).unwrap();
    assert_eq!(csv, format!("1.234,50;24/10/2026;1.234.567 ₫;0,000123456789;\"a;b\";{VI_NFC}\r\n"));
    s.execute("file.open", json!({"name": "vi.csv", "base64": saved["base64"]})).unwrap();
    assert_eq!(value(&s, "A1"), Value::Number(1234.5));
    assert_eq!(value(&s, "B1"), Value::Number(46319.0));
    assert_eq!(value(&s, "C1"), Value::Number(1_234_567.0));
    assert_eq!(value(&s, "D1"), Value::Number(0.000123456789));
    assert_eq!(value(&s, "E1"), Value::text("a;b"));
    assert_eq!(value(&s, "F1"), Value::text(VI_NFC));
    // en-US sessions are unchanged: `,` and `.`.
    let mut en = session();
    set(&mut en, "A1", "1234.5", None);
    set(&mut en, "B1", "x", None);
    let saved = en.execute("file.saveBytes", json!({"format": "csv"})).unwrap();
    assert_eq!(gridcraft_engine::io::base64_decode(saved["base64"].as_str().unwrap()).unwrap(), b"1234.5,x\r\n");
}

/// Sheets the engine adds are named in the interface's language (Trang tính n).
#[test]
fn generated_sheets_take_the_localized_prefix() {
    let mut s = session();
    s.sheet_name_prefix = "Trang tính".into();
    let mut rows = vec![json!(["Tháng", "Doanh thu"])];
    for t in 1..=12 {
        rows.push(json!([t, 10 + 2 * t]));
    }
    s.execute("range.setValues", json!({"range": "A1", "values": rows})).unwrap();
    s.execute("selection.set", json!({"range": "A1:B13"})).unwrap();
    let r = s.execute("data.forecastSheet", json!({"periods": 2})).unwrap();
    assert!(r["sheet"].as_str().unwrap().starts_with("Trang tính"), "{r}");
    s.execute("selection.set", json!({"range": "A1:B13"})).unwrap();
    s.execute("data.analysisToolPak", json!({"tool": "descriptive", "range": "B1:B13", "labels": true})).unwrap();
    s.execute("selection.set", json!({"range": "A1:B13"})).unwrap();
    s.execute("insert.pivotTable", json!({})).unwrap();
    let names: Vec<String> = s.active().unwrap().wb.sheets.iter().map(|sh| sh.name.clone()).collect();
    assert!(!names.iter().any(|n| n.starts_with("Sheet") && n != "Sheet1"), "{names:?}");
    assert!(names.iter().filter(|n| n.starts_with("Trang tính")).count() >= 3, "{names:?}");
}
