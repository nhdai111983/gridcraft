use gridcraft_core::{CellRef, DateSystem, Value};
use gridcraft_model::{Sheet, Workbook};

use crate::csv::{decode_text, sniff_delimiter, sniff_delimiter_in};
use crate::{CsvOptions, Format, IoError, read_csv, sniff, write_csv, write_csv_limited};

fn v(wb: &Workbook, r: u32, c: u32) -> Value {
    wb.sheet(0).unwrap().value(CellRef::new(r, c))
}

#[test]
fn quoting_and_newlines() {
    let src = "a,\"b,c\",\"say \"\"hi\"\"\"\r\n\"multi\nline\",,3\n\nlast";
    let wb = read_csv(src.as_bytes(), &CsvOptions::default()).unwrap();
    assert_eq!(wb.sheets[0].name, "Sheet1");
    assert_eq!(v(&wb, 0, 0), Value::text("a"));
    assert_eq!(v(&wb, 0, 1), Value::text("b,c"));
    assert_eq!(v(&wb, 0, 2), Value::text("say \"hi\""));
    assert_eq!(v(&wb, 1, 0), Value::text("multi\nline"));
    assert_eq!(v(&wb, 1, 1), Value::Empty);
    assert_eq!(v(&wb, 1, 2), Value::Number(3.0));
    assert_eq!(v(&wb, 2, 0), Value::Empty);
    assert_eq!(v(&wb, 3, 0), Value::text("last"));
}

#[test]
fn values_and_formats() {
    let src = "1,2.5,\"1,234\",50%,TRUE,#N/A,2026-03-04,$5.00,'007,text";
    let wb = read_csv(src.as_bytes(), &CsvOptions::default()).unwrap();
    assert_eq!(v(&wb, 0, 0), Value::Number(1.0));
    assert_eq!(v(&wb, 0, 1), Value::Number(2.5));
    assert_eq!(v(&wb, 0, 2), Value::Number(1234.0));
    assert_eq!(v(&wb, 0, 3), Value::Number(0.5));
    assert_eq!(v(&wb, 0, 4), Value::Bool(true));
    assert!(v(&wb, 0, 5).is_error());
    assert!(v(&wb, 0, 6).is_number());
    let s = wb.sheet(0).unwrap();
    let st = wb.styles.get(s.cell(CellRef::new(0, 6)).unwrap().style);
    assert_ne!(st.num_fmt.as_str(), "General");
    assert_eq!(v(&wb, 0, 8), Value::text("007"));
    // 1904 dates differ from 1900 dates.
    let wb2 = read_csv(b"2026-03-04", &CsvOptions { date_system: DateSystem::D1904, ..Default::default() }).unwrap();
    assert_eq!(wb2.date_system, DateSystem::D1904);
    assert_eq!(v(&wb, 0, 6).as_f64().unwrap() - v(&wb2, 0, 0).as_f64().unwrap(), 1462.0);
    // parse_values off keeps text.
    let wb3 = read_csv(b"1,TRUE", &CsvOptions { parse_values: false, ..Default::default() }).unwrap();
    assert_eq!(v(&wb3, 0, 0), Value::text("1"));
}

#[test]
fn encodings() {
    let mut b = vec![0xEF, 0xBB, 0xBF];
    b.extend_from_slice("é,x".as_bytes());
    assert_eq!(decode_text(&b), "é,x");
    let mut le = vec![0xFF, 0xFE];
    for u in "ü;1".encode_utf16() {
        le.extend_from_slice(&u.to_le_bytes());
    }
    assert_eq!(decode_text(&le), "ü;1");
    let mut be = vec![0xFE, 0xFF];
    for u in "ß".encode_utf16() {
        be.extend_from_slice(&u.to_be_bytes());
    }
    assert_eq!(decode_text(&be), "ß");
    assert_eq!(decode_text(&[0x80, b'a', 0xE9]), "€aé");
    let wb = read_csv(&le, &CsvOptions { delimiter: 0, ..Default::default() }).unwrap();
    assert_eq!(v(&wb, 0, 0), Value::text("ü"));
    assert_eq!(v(&wb, 0, 1), Value::Number(1.0));
}

#[test]
fn delimiter_sniffing() {
    assert_eq!(sniff_delimiter("a;b;c\n1;2;3\n", '"'), b';');
    assert_eq!(sniff_delimiter("a\tb\n1\t2\n", '"'), b'\t');
    assert_eq!(sniff_delimiter("a,b\n\"x;y;z\",2\n", '"'), b',');
    assert_eq!(sniff_delimiter("single", '"'), b',');
    // A delimiter missing from some lines is text, not a separator (single-column export round trip).
    assert_eq!(sniff_delimiter("alpha;beta\r\ngamma\r\n", '"'), b',');
    assert_eq!(sniff_delimiter("alpha\tbeta\r\ngamma\r\n", '"'), b',');
    assert_eq!(sniff_delimiter("a;b\r\n\r\nc;d\r\n", '"'), b';');
    let wb = read_csv(b"x;y\n1,5;2", &CsvOptions { delimiter: 0, ..Default::default() }).unwrap();
    assert_eq!(v(&wb, 1, 1), Value::Number(2.0));
    // The workbook remembers the separator it was read with, sniffed or given.
    assert_eq!(wb.csv_delimiter, Some(b';'));
    assert_eq!(read_csv(b"a\tb", &CsvOptions { delimiter: 0, ..Default::default() }).unwrap().csv_delimiter, Some(b'\t'));
    assert_eq!(read_csv(b"a;b", &CsvOptions::default()).unwrap().csv_delimiter, Some(b','));
    assert_eq!(Workbook::new().csv_delimiter, None);
    assert!(read_csv(b"a", &CsvOptions { delimiter: b'"', ..Default::default() }).is_err());
}

#[test]
fn writing() {
    let mut wb = Workbook::new();
    let mut s = Sheet::new("S");
    s.set_value(CellRef::new(1, 1), Value::text("a,b"));
    s.set_value(CellRef::new(1, 2), Value::text("q\"t"));
    s.set_value(CellRef::new(2, 1), Value::Number(0.1));
    s.set_value(CellRef::new(2, 2), Value::Bool(false));
    s.set_value(CellRef::new(3, 3), Value::text("x\ny"));
    s.set_value(CellRef::new(3, 1), Value::Error(gridcraft_core::CellError::Div0));
    wb.sheets = vec![std::sync::Arc::new(s)];
    // The caller supplies each field's text; quoting and line ends are the writer's.
    let sh = wb.sheet(0).unwrap();
    let raw = |c: CellRef| match sh.value(c) {
        Value::Empty => String::new(),
        Value::Number(n) => gridcraft_core::number_to_text(n),
        Value::Text(t) => t.to_string(),
        Value::Bool(b) => if b { "TRUE" } else { "FALSE" }.into(),
        Value::Error(e) => e.as_str().into(),
        Value::Array(_) => String::new(),
    };
    let out = String::from_utf8(write_csv(sh, b',', raw).unwrap()).unwrap();
    assert_eq!(out, "\"a,b\",\"q\"\"t\",\r\n0.1,FALSE,\r\n#DIV/0!,,\"x\ny\"\r\n");
    let semi = String::from_utf8(write_csv(sh, b';', raw).unwrap()).unwrap();
    assert!(semi.starts_with("a,b;"));
    assert!(write_csv(&Sheet::new("E"), b',', raw).unwrap().is_empty());
    // Round trip.
    let back = read_csv(out.as_bytes(), &CsvOptions::default()).unwrap();
    assert_eq!(v(&back, 0, 0), Value::text("a,b"));
    assert_eq!(v(&back, 2, 2), Value::text("x\ny"));
}

#[test]
fn sniffing_formats() {
    assert_eq!(sniff(b"PK\x03\x04rest"), Format::Xlsx);
    assert_eq!(sniff(b"a,b\n1,2"), Format::Csv);
    // An Excel password-protected file is a CFB/OLE2 compound file with an EncryptedPackage
    // stream; any other compound file is a legacy binary (.xls), not an encrypted workbook.
    assert_eq!(sniff(&crate::cfb::tests::build(&["EncryptionInfo", "EncryptedPackage"])), Format::Encrypted);
    assert_eq!(sniff(&crate::cfb::tests::build(&["Workbook"])), Format::LegacyBinary);
    assert_eq!(sniff(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0]), Format::LegacyBinary);
    assert_eq!(sniff(&[0xD0, 0xCF, 0x11, 0xE0, 0, 0, 1, 2]), Format::Unknown);
    assert_eq!(sniff(b""), Format::Unknown);
    assert_eq!(sniff(&[0xFF, 0xFE, b'a', 0]), Format::Csv);
}

#[test]
fn hostile_csv() {
    let _ = read_csv(&[0xFF; 1000], &CsvOptions::default()).unwrap();
    let _ = read_csv(b"\"unterminated,quote\n1,2", &CsvOptions::default()).unwrap();
    let wide = ",".repeat(20_000);
    let wb = read_csv(format!("{wide}x").as_bytes(), &CsvOptions::default()).unwrap();
    assert!(wb.sheet(0).unwrap().cells.is_empty(), "fields past column XFD are dropped");
}

/// A sheet with `a` in A1 and `b` in `corner`.
fn two_cells(corner: (u32, u32)) -> Sheet {
    let mut s = Sheet::new("S");
    s.set_value(CellRef::new(0, 0), Value::text("a"));
    s.set_value(CellRef::new(corner.0, corner.1), Value::text("b"));
    s
}

fn text_of(sh: &Sheet) -> impl FnMut(CellRef) -> String + '_ {
    |c| match sh.value(c) {
        Value::Text(t) => t.to_string(),
        _ => String::new(),
    }
}

#[test]
fn two_far_apart_cells_are_refused_before_anything_is_allocated() {
    // A1 and XFD1048576 span the whole grid: 16 GiB of commas. Refused at once, with the numbers.
    let s = two_cells((gridcraft_core::MAX_ROWS - 1, gridcraft_core::MAX_COLS - 1));
    let err = write_csv(&s, b',', text_of(&s)).unwrap_err();
    assert!(matches!(err, IoError::TooLarge(_)), "{err}");
    let msg = err.to_string();
    assert!(msg.contains("1048576 rows by 16384 columns") && msg.contains("512 MiB"), "{msg}");
    // A single row or column across the grid is refused too when it is over the cap.
    let wide = two_cells((gridcraft_core::MAX_ROWS - 1, 0));
    assert!(write_csv_limited(&wide, b',', 1 << 20, text_of(&wide)).is_err());
}

#[test]
fn the_cap_counts_the_bytes_the_cells_hold_and_accepted_exports_keep_every_position() {
    // 3 rows x 3 columns, 14 bytes: accepted at the cap, refused one byte under.
    let s = two_cells((2, 2));
    let ok = write_csv_limited(&s, b',', 14, text_of(&s)).unwrap();
    assert_eq!(String::from_utf8(ok).unwrap(), "a,,\r\n,,\r\n,,b\r\n");
    assert!(write_csv_limited(&s, b',', 13, text_of(&s)).is_err());
    // Rows that pass the lower bound but hold long text stop as soon as the cap is passed.
    let mut long = Sheet::new("L");
    for r in 0..50u32 {
        long.set_value(CellRef::new(r, 0), Value::text("x".repeat(100)));
    }
    let err = write_csv_limited(&long, b',', 1000, text_of(&long)).unwrap_err();
    assert!(err.to_string().contains("stopped at row"), "{err}");
    assert!(write_csv_limited(&long, b',', 50 * 102, text_of(&long)).is_ok());
    // A tab is one byte, like a comma.
    let tabs = write_csv_limited(&s, b'\t', 14, text_of(&s)).unwrap();
    assert_eq!(tabs.len(), 14);
}

/// NexusOffice: a CSV read in vi-VN takes its values the vi-VN way, and `;` (the list separator
/// there) as the delimiter whenever it appears; commas that only sit between digits are decimal
/// commas, not field separators. en-US reading is unchanged.
#[test]
fn csv_in_vietnamese() {
    use gridcraft_core::Locale::{EnUs, ViVn};
    let vi = CsvOptions { delimiter: 0, locale: ViVn, ..Default::default() };
    let wb = read_csv("1.234,5;24/10/2026;1.234 ₫;Hà Nội\r\n".as_bytes(), &vi).unwrap();
    let sh = &wb.sheets[0];
    assert_eq!(sh.value(CellRef::new(0, 0)), Value::Number(1234.5));
    assert_eq!(sh.value(CellRef::new(0, 1)), Value::Number(46319.0));
    assert_eq!(sh.value(CellRef::new(0, 2)), Value::Number(1234.0));
    assert_eq!(sh.value(CellRef::new(0, 3)), Value::Text("Hà Nội".into()));
    assert_eq!(wb.csv_delimiter, Some(b';'));
    // One column of decimal-comma numbers is not split at the commas.
    let wb = read_csv(b"1,5\n2,25\n", &vi).unwrap();
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 0)), Value::Number(2.25));
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 1)), Value::Empty);
    // Commas between words still separate fields (an en-US export).
    let wb = read_csv("Tên,Số\nAn,5\n".as_bytes(), &vi).unwrap();
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 1)), Value::Number(5.0));
    assert_eq!(sniff_delimiter_in("1,5\n", '"', EnUs), b',');
    assert_eq!(sniff_delimiter_in("\"a;b\",1\n", '"', ViVn), b',');
    // A `;` on one line only doesn't outvote a consistent comma (or tab) layout.
    assert_eq!(sniff_delimiter_in("a,b,c\nx;y,1,2\nd,e,f\n", '"', ViVn), b',');
    assert_eq!(sniff_delimiter_in("a\tb\nc;d\te\n", '"', ViVn), b'\t');
    // `;` on every line wins over more decimal commas.
    assert_eq!(sniff_delimiter_in("1,5;2,5;3,5\n4,5;5,5;6,5\n", '"', ViVn), b';');
    // A title line without `;` above a `;` table; a header and whole numbers in a decimal-comma column.
    let wb = read_csv("Báo cáo\nTên;Giá\nAn;1,5\n".as_bytes(), &vi).unwrap();
    assert_eq!(wb.csv_delimiter, Some(b';'));
    assert_eq!(wb.sheets[0].value(CellRef::new(2, 1)), Value::Number(1.5));
    let wb = read_csv("Giá\n1,5\n2\n".as_bytes(), &vi).unwrap();
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 0)), Value::Number(1.5));
    assert_eq!(wb.sheets[0].value(CellRef::new(2, 0)), Value::Number(2.0));
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 1)), Value::Empty);
    // Quoted fields, a single row, an empty file.
    assert_eq!(sniff_delimiter_in("\"a,b\";\"c\"\n", '"', ViVn), b';');
    assert_eq!(sniff_delimiter_in("a;b", '"', ViVn), b';');
    assert_eq!(sniff_delimiter_in("", '"', ViVn), sniff_delimiter("", '"'));
    // Several commas per line, all between digits: an en-US numeric export, split at the commas.
    let wb = read_csv(b"1,2,3\n4,5,6\n", &vi).unwrap();
    assert_eq!(wb.sheets[0].value(CellRef::new(1, 2)), Value::Number(6.0));
    assert_eq!(wb.csv_delimiter, Some(b','), "saving it keeps the commas");
    let en = CsvOptions { delimiter: 0, ..Default::default() };
    assert_eq!(read_csv(b"1.5,2\n", &en).unwrap().sheets[0].value(CellRef::new(0, 0)), Value::Number(1.5));
}

/// NexusOffice (vi-VN sniffer): a title line with no separator above a comma or tab table, with
/// an unquoted `;` inside a text cell, splits at the commas (tabs), not at the stray `;`.
#[test]
fn csv_in_vietnamese_title_line_and_stray_semicolon() {
    use gridcraft_core::Locale::{EnUs, ViVn};
    assert_eq!(sniff_delimiter_in("Report\nName,Note\nA,x; y\n", '"', ViVn), b',');
    assert_eq!(sniff_delimiter_in("Report\r\n\r\nName,Note\r\nA,x; y", '"', ViVn), b',');
    assert_eq!(sniff_delimiter_in("Report\nName\tNote\nA\tx; y\n", '"', ViVn), b'\t');
    assert_eq!(sniff_delimiter_in("Báo cáo\nBáo cáo tháng 10\nTên,Ghi chú\nAn,x; y\nBình,z\n", '"', ViVn), b',');
    let vi = CsvOptions { delimiter: 0, locale: ViVn, ..Default::default() };
    let wb = read_csv(b"Report\nName,Note\nA,x; y\n", &vi).unwrap();
    assert_eq!(wb.csv_delimiter, Some(b','));
    assert_eq!(wb.sheets[0].value(CellRef::new(0, 0)), Value::Text("Report".into()));
    assert_eq!(wb.sheets[0].value(CellRef::new(2, 1)), Value::Text("x; y".into()));
    let wb = read_csv(b"Report\nName\tNote\nA\tx; y\n", &vi).unwrap();
    assert_eq!(wb.sheets[0].value(CellRef::new(2, 1)), Value::Text("x; y".into()));
    // The cases a title line used to decide still read the same way.
    assert_eq!(sniff_delimiter_in("Báo cáo\nTên;Giá\nAn;1,5\n", '"', ViVn), b';');
    assert_eq!(sniff_delimiter_in("Báo cáo, tháng 10\nTên;Giá\nAn;1,5\n", '"', ViVn), b';');
    assert_eq!(sniff_delimiter_in("Giá\n1,5\n2\n", '"', ViVn), b';');
    assert_eq!(sniff_delimiter_in("Tên\nAn\nBình\n", '"', ViVn), sniff_delimiter("Tên\nAn\nBình\n", '"'));
    // en-US is unchanged.
    for t in ["Report\nName,Note\nA,x; y\n", "Report\nName\tNote\nA\tx; y\n", "a;b\n"] {
        assert_eq!(sniff_delimiter_in(t, '"', EnUs), sniff_delimiter(t, '"'), "{t:?}");
    }
    // Hostile text never panics.
    for t in ["\n\n\n", "\"", "\"\n;", ";\n,\n\t", "\u{0},;\t\"", &"x\n".repeat(50), &";,".repeat(40_000)] {
        let _ = sniff_delimiter_in(t, '"', ViVn);
        let _ = read_csv(t.as_bytes(), &vi);
    }
}
