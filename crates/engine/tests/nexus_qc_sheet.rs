//! NexusOffice QC wave 1 (QC-SHT-*) engine behaviour.

use gridcraft_core::{CellRef, Locale};
use gridcraft_engine::Session;
use serde_json::json;

fn width(s: &Session, col: u32) -> f32 {
    s.doc().unwrap().wb.active().unwrap().col_width(col)
}

/// QC-SHT-10: a date or formatted number typed into a default-width column widens it, as in
/// Excel, instead of showing #######. Only when typed (`typed`), not for General numbers, text
/// or formulas, and never over a width someone set.
#[test]
fn typed_date_or_formatted_number_widens_a_default_column() {
    let mut s = Session::new();
    s.new_workbook();
    s.set_locale(Locale::ViVn);
    let default = width(&s, 2);
    let typed = |s: &mut Session, cell: &str, input: &str| {
        s.execute("cell.set", json!({"cell": cell, "input": input, "locale": "vi-VN", "typed": true})).unwrap();
    };
    typed(&mut s, "C2", "26/10/2026");
    let after_date = width(&s, 2);
    assert!(after_date > default, "date widened C: {default} -> {after_date}");
    let text = gridcraft_engine::display::value_text(&s.doc().unwrap().wb, s.doc().unwrap().wb.active().unwrap(), CellRef::parse("C2").unwrap());
    assert_eq!(text, "26/10/2026");
    typed(&mut s, "D2", "2.000.000.000");
    assert!(width(&s, 3) > default, "a formatted number widened D");
    // Undo puts the width back with the value.
    s.execute("edit.undo", json!({})).unwrap();
    assert_eq!(width(&s, 3), default);

    // General numbers, text and formulas leave the width alone.
    typed(&mut s, "E2", "123456789012");
    typed(&mut s, "F2", "một chuỗi chữ rất dài không làm giãn cột");
    typed(&mut s, "G2", "=DATE(2026,10,26)");
    for col in [4, 5, 6] {
        assert_eq!(width(&s, col), default, "column {col}");
    }
    // A width set by hand stays (Excel shows ### there).
    s.execute("home.columnWidth", json!({"cols": "H:H", "width": 20})).unwrap();
    let set = width(&s, 7);
    typed(&mut s, "H2", "26/10/2026");
    assert_eq!(width(&s, 7), set);
    // Not typed (API, paste, MCP): unchanged, as before.
    s.execute("cell.set", json!({"cell": "I2", "input": "26/10/2026", "locale": "vi-VN"})).unwrap();
    assert_eq!(width(&s, 8), default);
    // A value that already fits leaves it alone.
    typed(&mut s, "J2", "5%");
    assert_eq!(width(&s, 9), default);
}

fn shown(s: &Session, a1: &str) -> String {
    let wb = &s.doc().unwrap().wb;
    gridcraft_engine::display::value_text(wb, wb.active().unwrap(), CellRef::parse(a1).unwrap())
}

/// QC-SHT-12: a formula typed into a General cell shows its result in the format of the first
/// cell it reads, as in Excel (SUM of 1.234,50 … shows 2.001.979,25, not 2001979).
#[test]
fn typed_formula_takes_the_format_of_what_it_reads() {
    let mut s = Session::new();
    s.new_workbook();
    s.set_locale(Locale::ViVn);
    let typed = |s: &mut Session, cell: &str, input: &str| {
        s.execute("cell.set", json!({"cell": cell, "input": input, "locale": "vi-VN", "typed": true})).unwrap();
    };
    for (cell, v) in [("B2", "1.234,50"), ("B3", "2.000.000,00"), ("B4", "500,25"), ("B5", "244,50")] {
        typed(&mut s, cell, v);
    }
    typed(&mut s, "B6", "=SUM(B2:B5)");
    assert_eq!(shown(&s, "B6"), "2.001.979,25");
    typed(&mut s, "C6", "=B2*2");
    assert_eq!(shown(&s, "C6"), "2.469,00");
    // Counting keeps General; so does a cell someone formatted, and text references.
    typed(&mut s, "D6", "=COUNT(B2:B5)");
    assert_eq!(shown(&s, "D6"), "4");
    s.execute("home.numberFormat", json!({"range": "E6", "code": "0.0"})).unwrap();
    typed(&mut s, "E6", "=SUM(B2:B5)");
    assert_eq!(shown(&s, "E6"), "2001979,3");
    // Not typed (API): unchanged, as before.
    s.execute("cell.set", json!({"cell": "F6", "input": "=SUM(B2:B5)", "locale": "vi-VN"})).unwrap();
    assert_eq!(shown(&s, "F6"), "2001979,25");
}

/// QC-SHT-05: a table without a header row gets headers in the region's language (Cột1, Cột2).
#[test]
fn table_headers_follow_the_region() {
    for (locale, want) in [(Locale::ViVn, "Cột1"), (Locale::EnUs, "Column1")] {
        let mut s = Session::new();
        s.new_workbook();
        s.set_locale(locale);
        s.execute("range.setValues", json!({"range": "A1", "values": [[1, 2], [3, 4]]})).unwrap();
        s.execute("insert.table", json!({"range": "A1:B2", "header": false})).unwrap();
        assert_eq!(shown(&s, "A1"), want);
    }
}
