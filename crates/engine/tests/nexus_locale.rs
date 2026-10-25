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
    assert_eq!(value(&s, "B2"), Value::text(decomposed.as_str()));
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
    // `#,##0.00` is built-in 4; đồng's code is written en-US.
    assert!(styles.contains("numFmtId=\"4\"") && styles.contains("formatCode=\"#,##0 [$₫-42A]\""), "{styles}");
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

/// Text pasted from another app is read in the regional format the paste names, like typing.
#[test]
fn pasted_text_reads_the_regional_format() {
    let mut s = session();
    s.execute("edit.paste", json!({"at": "A1", "text": "1.234\t1.234,5\t24/10/2026\t0.125\n", "locale": "vi-VN"})).unwrap();
    assert_eq!(value(&s, "A1"), Value::Number(1234.0));
    assert_eq!(value(&s, "B1"), Value::Number(1234.5));
    assert_eq!(value(&s, "C1"), Value::Number(46319.0));
    assert_eq!(value(&s, "D1"), Value::text("0.125"));
    // Without `locale` the API pastes en-US.
    s.execute("edit.paste", json!({"at": "A2", "text": "1.234\t1.234,5\n"})).unwrap();
    assert_eq!(value(&s, "A2"), Value::Number(1.234));
    assert_eq!(value(&s, "B2"), Value::text("1.234,5"));
}

/// Values typed in dialogs (filter, scenario, solver, forecast, From Text) read the regional format
/// and what the workbook stores stays en-US.
#[test]
fn dialog_values_read_the_regional_format_and_store_en_us() {
    let mut s = session();
    s.execute("range.setValues", json!({"range": "A1", "values": [["Giá"], [1.2], [1.6], [2.5]]})).unwrap();
    s.execute("data.filter", json!({"range": "A1:A4"})).unwrap();
    s.execute("data.filterBy", json!({"column": "A", "custom": {"op": ">", "value": "1,5"}, "locale": "vi-VN"})).unwrap();
    let sh = s.active().unwrap().wb.active().unwrap().clone();
    assert!(sh.is_row_hidden(1) && !sh.is_row_hidden(2) && !sh.is_row_hidden(3), "only 1.2 is hidden");
    let stored = format!("{:?}", sh.autofilter.as_ref().unwrap().criteria);
    assert!(stored.contains("\"1.5\"") && !stored.contains("1,5"), "{stored}");

    let mut s = session();
    s.execute("range.setValues", json!({"range": "B2", "values": [[1], [2]]})).unwrap();
    s.execute("data.scenarioAdd", json!({"name": "Thấp", "changing": "B2:B3", "values": ["1,5", "1,234.5"], "locale": "vi-VN"})).unwrap();
    s.execute("data.scenarioShow", json!({"name": "Thấp"})).unwrap();
    assert_eq!(value(&s, "B2"), Value::Number(1.5));
    assert_eq!(value(&s, "B3"), Value::text("1,234.5"), "text in vi-VN stays text when shown");

    let mut s = session();
    set(&mut s, "A1", "1", None);
    set(&mut s, "B1", "=A1*2", None);
    let r = s
        .execute(
            "data.solver",
            json!({"objective": "B1", "goal": "max", "variables": "A1", "constraints": ["A1 <= 2,5"], "method": "simplex", "locale": "vi-VN"}),
        )
        .unwrap();
    assert_eq!(r["status"], "optimal", "{r}");
    assert!((value(&s, "A1").as_f64().unwrap() - 2.5).abs() < 1e-9);

    let mut s = session();
    let r = s.execute("data.getData", json!({"text": "Số;Ngày\n1,5;24/10/2026\n", "locale": "vi-VN"})).unwrap();
    assert!(r.is_object(), "{r}");
    let sh = s.active().unwrap().wb.active().unwrap().clone();
    let found: Vec<Value> = sh.cells.iter().map(|(_, c)| c.value.clone()).collect();
    assert!(found.contains(&Value::Number(1.5)) && found.contains(&Value::Number(46319.0)), "{found:?}");
}

/// A workbook opened in a vi-VN session is recalculated in it when a formula depends on the
/// locale (its cached results are the file's en-US ones), and stays saved.
#[test]
fn opening_in_vietnamese_recalculates_locale_dependent_formulas() {
    let mut en = session();
    set(&mut en, "A1", "=TEXT(1234.5,\"#.##0,00\")", None);
    let en_value = value(&en, "A1");
    let saved = en.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let mut s = Session::new();
    s.set_locale(Locale::ViVn);
    s.execute("file.open", json!({"name": "en.xlsx", "base64": saved["base64"]})).unwrap();
    assert_eq!(value(&s, "A1"), Value::text("1.234,50"));
    assert_ne!(value(&s, "A1"), en_value);
    assert!(!s.active().unwrap().is_dirty());
}

/// The API's display can be en-US while the workbook shows vi-VN (the control channel's default).
#[test]
fn display_locale_can_be_overridden_for_the_api() {
    let mut s = session();
    set(&mut s, "A1", "1234.5", None);
    s.set_locale(Locale::ViVn);
    assert_eq!(text(&s, "A1"), "1234,5");
    let shown = gridcraft_engine::display::with_locale(Locale::EnUs, || text(&s, "A1"));
    assert_eq!(shown, "1234.5");
    let csv = gridcraft_engine::display::with_locale(Locale::EnUs, || s.active().map(|d| gridcraft_engine::io::save_bytes(&d.wb, "x.csv").unwrap()));
    assert_eq!(csv.unwrap(), b"1234.5\r\n");
    assert_eq!(text(&s, "A1"), "1234,5", "the override ends with the call");
}

/// An API caller reading in en-US (the control channel's default) never changes what is written
/// to the user's file: saving a vi-VN CSV keeps `1,5;24/10/2026`. Only the bytes returned to the
/// caller (`file.saveBytes`) are in the caller's locale.
#[test]
fn saving_the_users_csv_keeps_the_workbooks_locale() {
    let dir = std::env::temp_dir().join(format!("nexus-vi-csv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("a.csv");
    std::fs::write(&path, "1,5;24/10/2026\r\n").unwrap();
    let mut s = Session::new();
    s.set_locale(Locale::ViVn);
    s.execute("file.open", json!({"path": path.to_str().unwrap()})).unwrap();
    assert_eq!(value(&s, "A1"), Value::Number(1.5));
    let returned = gridcraft_engine::display::with_locale(Locale::EnUs, || {
        s.execute("file.save", json!({"path": path.to_str().unwrap()})).unwrap();
        s.execute("file.saveBytes", json!({"format": "csv"})).unwrap()
    });
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "1,5;24/10/2026\r\n", "the user's file stays vi-VN");
    let bytes = gridcraft_engine::io::base64_decode(returned["base64"].as_str().unwrap()).unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), "1.5;10/24/2026\r\n", "the caller reads en-US");
    // Reopened in vi-VN, both cells are still numbers.
    let mut again = Session::new();
    again.set_locale(Locale::ViVn);
    again.execute("file.open", json!({"path": path.to_str().unwrap()})).unwrap();
    assert_eq!(value(&again, "A1"), Value::Number(1.5));
    assert_eq!(value(&again, "B1"), Value::Number(46319.0));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Filters evaluate in the workbook's regional format, whoever runs them: the same `>1,5`
/// criteria cell selects the same rows through the API (en-US display) as in the window.
#[test]
fn advanced_filter_evaluates_in_the_workbooks_locale() {
    let mut s = session();
    s.execute("range.setValues", json!({"range": "A1", "values": [["Giá"], [1.2], [1.6], [2.5]]})).unwrap();
    s.execute("range.setValues", json!({"range": "H1", "values": [["Giá"], [">1,5"]]})).unwrap();
    s.set_locale(Locale::ViVn);
    let ui = s.execute("data.advancedFilter", json!({"range": "A1:A4", "criteria": "H1:H2"})).unwrap();
    let api = gridcraft_engine::display::with_locale(Locale::EnUs, || {
        s.execute("data.advancedFilter", json!({"range": "A1:A4", "criteria": "H1:H2"})).unwrap()
    });
    assert_eq!(ui["matched"], 2);
    assert_eq!(api["matched"], ui["matched"]);
}

/// Solver's "Value Of" target typed in vi-VN.
#[test]
fn solver_value_of_reads_the_regional_format() {
    let mut s = session();
    set(&mut s, "A1", "1", None);
    set(&mut s, "B1", "=A1*2", None);
    let r = s
        .execute("data.solver", json!({"objective": "B1", "goal": "value", "value": "2,5", "variables": "A1", "method": "grg", "locale": "vi-VN"}))
        .unwrap();
    assert!((value(&s, "B1").as_f64().unwrap() - 2.5).abs() < 1e-6, "{r}");
    assert!(
        s.execute("data.solver", json!({"objective": "B1", "goal": "value", "value": "2,5", "variables": "A1"})).is_err(),
        "en-US reads `2,5` as no number"
    );
}

/// Forecast Sheet dates, Text to Columns parts and From Text/CSV values in vi-VN.
#[test]
fn more_entry_points_read_the_regional_format() {
    let mut s = session();
    let mut rows = vec![json!(["Ngày", "Doanh thu"])];
    for t in 0..16 {
        rows.push(json!([46300 + t, 10 + 2 * t]));
    }
    s.execute("range.setValues", json!({"range": "A1", "values": rows})).unwrap();
    s.execute("selection.set", json!({"range": "A1:B17"})).unwrap();
    // The last date is 46315 (2026-10-20): end 24/10/2026 is 4 periods on.
    let r = s.execute("data.forecastSheet", json!({"end": "24/10/2026", "locale": "vi-VN"})).unwrap();
    assert_eq!(r["periods"], 4, "{r}");

    let mut s = session();
    s.execute("range.setValues", json!({"range": "A1", "values": [["1,5;2.500;24/10/2026"]]})).unwrap();
    s.execute("data.textToColumns", json!({"range": "A1", "delimiters": [";"], "locale": "vi-VN"})).unwrap();
    assert_eq!([value(&s, "A1"), value(&s, "B1"), value(&s, "C1")], [Value::Number(1.5), Value::Number(2500.0), Value::Number(46319.0)]);

    let mut s = session();
    s.execute("data.fromTextCsv", json!({"text": "Số;Ngày\n1,5;24/10/2026\n", "locale": "vi-VN"})).unwrap();
    let sh = s.active().unwrap().wb.active().unwrap().clone();
    assert_eq!(sh.value(CellRef::parse("A2").unwrap()), Value::Number(1.5));
    assert_eq!(sh.value(CellRef::parse("B2").unwrap()), Value::Number(46319.0));
    let e = s.execute("data.fromTextCsv", json!({"text": "[1,", "locale": "vi-VN"})).unwrap_err().to_string();
    assert!(e.contains("data.fromTextCsv"), "{e}");
}
