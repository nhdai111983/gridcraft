//! `#SPILL!` and `#CALC!` exist only as formula results: the XLSX format has no error code for
//! them (the writer caches a formula showing one as `#VALUE!`, and reopening recalculates it).
//! A cell that holds one as a constant (pasted as values, or typed) could not be saved as it is,
//! so it holds `#VALUE!` from the start. Found by the command-sequence fuzz (seed 79, budget).

use gridcraft_core::{CellError, CellRef, Value};
use gridcraft_engine::calc::Calc;
use gridcraft_engine::{Session, io};
use serde_json::json;

fn value(s: &Session, a1: &str) -> Value {
    let d = s.doc().unwrap();
    d.wb.active().unwrap().value(CellRef::parse(a1).unwrap())
}

fn reloaded(s: &Session, a1: &str) -> Value {
    let wb = &s.doc().unwrap().wb;
    let bytes = io::save_bytes(wb, "t.xlsx").unwrap();
    let (mut back, _) = io::open_bytes("t.xlsx", &bytes).unwrap();
    Calc::new().recalc_all(&mut back);
    back.active().unwrap().value(CellRef::parse(a1).unwrap())
}

#[test]
fn pasting_a_blocked_spill_as_values_keeps_what_the_file_keeps() {
    let mut s = Session::new();
    s.execute("file.new", json!({"sample": "budget"})).unwrap();
    // UNIQUE spills right into F11:G11, which hold values: E11 shows #SPILL!.
    s.execute("range.setValues", json!({"range": "E11", "values": [["=UNIQUE(E15:H20)", "70", 200]]})).unwrap();
    assert_eq!(value(&s, "E11"), Value::Error(CellError::Spill));
    s.execute("selection.set", json!({"cell": "A1"})).unwrap();
    s.execute("edit.copy", json!({"range": "D11:F15"})).unwrap();
    s.execute("edit.pasteSpecial", json!({"what": "values", "transpose": true})).unwrap();
    // Transposed, E11 (row 1, column 2 of the copy) lands on A2 as a constant.
    let a2 = value(&s, "A2");
    assert_eq!(a2, Value::Error(CellError::Value));
    assert_eq!(reloaded(&s, "A2"), a2);
    // The formula itself still shows #SPILL! and still reopens as #SPILL!.
    assert_eq!(reloaded(&s, "E11"), Value::Error(CellError::Spill));
}

#[test]
fn typed_formula_only_errors_keep_what_the_file_keeps() {
    for typed in ["#SPILL!", "#CALC!", "#spill!"] {
        let mut s = Session::new();
        s.new_workbook();
        s.execute("cell.set", json!({"cell": "B2", "input": typed})).unwrap();
        let v = value(&s, "B2");
        assert_eq!(v, Value::Error(CellError::Value), "{typed}");
        assert_eq!(reloaded(&s, "B2"), v, "{typed}");
    }
    // The errors the file format has stay as typed.
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "B2", "input": "#N/A"})).unwrap();
    assert_eq!(value(&s, "B2"), Value::Error(CellError::NA));
    assert_eq!(reloaded(&s, "B2"), Value::Error(CellError::NA));
}
