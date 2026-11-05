//! Two dynamic arrays whose spill areas collide: which one spills must not depend on the order
//! the formulas were entered or calculated in. A full recalculation (F9, opening the file) lays
//! the arrays out in row-major order of their anchors, so the first anchor spills and the later
//! one is `#SPILL!`; an incremental recalculation after each edit must give the same result.
//!
//! Found by the command-sequence fuzz (seed 98, grades sample): `cell.set C22 =UNIQUE(K1:L3)`
//! then `range.fill D18:G20 =C8:C11` left C22 spilling and D20 `#SPILL!`, while a full
//! recalculation spilled D20 and blocked C22.

use gridcraft_core::{CellError, CellRef, Value};
use gridcraft_engine::Session;
use gridcraft_engine::calc::Calc;
use serde_json::json;

fn value(s: &Session, address: &str) -> Value {
    s.doc().unwrap().wb.active().unwrap().value(CellRef::parse(address).unwrap())
}

/// Every value of the active sheet as an incremental recalculation left it, against a full
/// recalculation from scratch.
fn assert_same_as_full_recalc(s: &Session, when: &str) {
    let wb = s.doc().unwrap().wb.clone();
    let mut fresh = (*wb).clone();
    Calc::new().recalc_all(&mut fresh);
    let (a, b) = (wb.active().unwrap(), fresh.active().unwrap());
    for row in 0..30 {
        for col in 0..12 {
            let c = CellRef::new(row, col);
            assert_eq!(a.value(c), b.value(c), "{when}: {c} differs from a full recalculation");
        }
    }
}

const SPILL: Value = Value::Error(CellError::Spill);

/// D1 spills down D1:D3 and C3 spills across C3:E3; they meet at D3. D1 comes first in
/// row-major order, so it spills and C3 is blocked, whichever was typed first.
#[test]
fn the_first_anchor_in_row_major_order_spills_whatever_the_entry_order() {
    for order in [["C3", "D1"], ["D1", "C3"]] {
        let mut s = Session::new();
        s.new_workbook();
        for cell in order {
            let input = if cell == "D1" { "=SEQUENCE(3)" } else { "=SEQUENCE(1,3)" };
            s.execute("cell.set", json!({"cell": cell, "input": input})).unwrap();
            assert_same_as_full_recalc(&s, &format!("{order:?} after {cell}"));
        }
        assert_eq!(value(&s, "D1"), Value::Number(1.0), "{order:?}");
        assert_eq!(value(&s, "D3"), Value::Number(3.0), "{order:?}");
        assert_eq!(value(&s, "C3"), SPILL, "{order:?}");
        assert_eq!(value(&s, "E3"), Value::Empty, "{order:?}");

        // Clearing the winner lets the other one spill.
        s.execute("cell.set", json!({"cell": "D1", "input": ""})).unwrap();
        assert_same_as_full_recalc(&s, &format!("{order:?} after clearing D1"));
        assert_eq!(value(&s, "C3"), Value::Number(1.0), "{order:?}");
        assert_eq!(value(&s, "E3"), Value::Number(3.0), "{order:?}");

        // Undo puts D1 back, and it spills again over C3's area.
        s.execute("edit.undo", json!({})).unwrap();
        assert_eq!(value(&s, "D3"), Value::Number(3.0), "{order:?} undo");
        assert_eq!(value(&s, "C3"), SPILL, "{order:?} undo");
    }
}

/// The fuzz case: a fill enters D20 (=C10:C13, spilling D20:D23) over the area C22's UNIQUE
/// already spills into (C22:D24). D20 comes first in row-major order, so it takes the area.
#[test]
fn a_filled_formula_above_takes_the_area_from_a_spill_below() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "K1:L3", "values": [[1, 2], [3, 4], [5, 6]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C22", "input": "=UNIQUE(K1:L3)"})).unwrap();
    assert_eq!(value(&s, "D23"), Value::Number(4.0));
    assert_same_as_full_recalc(&s, "after C22");
    s.execute("range.fill", json!({"range": "D18:G20", "input": "=C8:C11"})).unwrap();
    assert_same_as_full_recalc(&s, "after the fill");
    assert_eq!(value(&s, "C22"), SPILL);
    assert_eq!(value(&s, "D23"), Value::Number(0.0));
}
