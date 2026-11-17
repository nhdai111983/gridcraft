//! Two dynamic arrays whose spill areas collide: the result must not depend on the order the
//! formulas were *calculated* in, and an incremental recalculation must agree with a full one
//! (F9, opening the file).
//!
//! The rule, measured against Excel upstream (storytold/gridcraft#2483, documented on `Standing`
//! in `crates/calc/src/recalc.rs`): the array that already holds its spill keeps the cells,
//! whichever anchor comes first; the other shows `#SPILL!` until the holder is cleared. With no
//! layout to remember (a first calculation), the anchor first in reading order wins. So the
//! entry order decides the winner here, while the calculation order never does.
//!
//! These cases come from the command-sequence fuzz (seed 98, grades sample): `cell.set C22
//! =UNIQUE(K1:L3)` then `range.fill D18:G20 =C8:C11` once left C22 spilling in memory and D20
//! `#SPILL!`, while a full recalculation gave the cells to D20 instead.

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

/// D1 spills down D1:D3 and C3 spills across C3:E3; they meet at D3. Whichever is entered first
/// holds those cells, the other is `#SPILL!`, and clearing the holder lets it spill — the same
/// either way round, and the same as a full recalculation at every step.
#[test]
fn the_spill_that_already_holds_its_cells_keeps_them_in_either_entry_order() {
    for order in [["C3", "D1"], ["D1", "C3"]] {
        let mut s = Session::new();
        s.new_workbook();
        for cell in order {
            let input = if cell == "D1" { "=SEQUENCE(3)" } else { "=SEQUENCE(1,3)" };
            s.execute("cell.set", json!({"cell": cell, "input": input})).unwrap();
            assert_same_as_full_recalc(&s, &format!("{order:?} after {cell}"));
        }
        // The one entered first spilled before the other was there, so it keeps D3.
        if order[0] == "D1" {
            assert_eq!(value(&s, "D1"), Value::Number(1.0), "{order:?}");
            assert_eq!(value(&s, "D3"), Value::Number(3.0), "{order:?}");
            assert_eq!(value(&s, "C3"), SPILL, "{order:?}");
            assert_eq!(value(&s, "E3"), Value::Empty, "{order:?}");
        } else {
            assert_eq!(value(&s, "C3"), Value::Number(1.0), "{order:?}");
            assert_eq!(value(&s, "D3"), Value::Number(2.0), "{order:?}");
            assert_eq!(value(&s, "E3"), Value::Number(3.0), "{order:?}");
            assert_eq!(value(&s, "D1"), SPILL, "{order:?}");
        }

        // Clearing the holder lets the blocked one spill.
        let (holder, blocked) = if order[0] == "D1" { ("D1", "C3") } else { ("C3", "D1") };
        s.execute("cell.set", json!({"cell": holder, "input": ""})).unwrap();
        assert_same_as_full_recalc(&s, &format!("{order:?} after clearing {holder}"));
        assert_eq!(value(&s, blocked), Value::Number(1.0), "{order:?} after clearing {holder}");
        assert_eq!(value(&s, "D3"), Value::Number(if blocked == "D1" { 3.0 } else { 2.0 }), "{order:?}");

        // Undo puts the holder back as it was, spill and all, so it holds the cells again.
        s.execute("edit.undo", json!({})).unwrap();
        assert_same_as_full_recalc(&s, &format!("{order:?} after undo"));
        assert_eq!(value(&s, holder), Value::Number(1.0), "{order:?} undo: the holder is back");
        assert_eq!(value(&s, blocked), SPILL, "{order:?} undo");
        assert_eq!(value(&s, "D3"), Value::Number(if holder == "D1" { 3.0 } else { 2.0 }), "{order:?} undo");
    }
}

/// The fuzz case: a fill enters D20 (`=C8:C11`, which would spill D20:D23) over the area C22's
/// UNIQUE already spills into (C22:D24). C22 holds those cells, so the filled formulas that
/// reach into them are `#SPILL!` and C22 is left alone.
#[test]
fn a_spill_keeps_its_area_against_a_later_fill_above_it() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "K1:L3", "values": [[1, 2], [3, 4], [5, 6]]})).unwrap();
    s.execute("cell.set", json!({"cell": "C22", "input": "=UNIQUE(K1:L3)"})).unwrap();
    assert_eq!(value(&s, "D23"), Value::Number(4.0));
    assert_same_as_full_recalc(&s, "after C22");
    s.execute("range.fill", json!({"range": "D18:G20", "input": "=C8:C11"})).unwrap();
    assert_same_as_full_recalc(&s, "after the fill");
    // C22's spill is untouched, and the filled formulas reaching into it are blocked.
    assert_eq!(value(&s, "C22"), Value::Number(1.0));
    assert_eq!(value(&s, "D23"), Value::Number(4.0));
    assert_eq!(value(&s, "D20"), SPILL);
    assert_eq!(value(&s, "D19"), SPILL);
    // D18 is `#SPILL!` too, for the ordinary reason: its own area (D18:D21) holds the formulas
    // the same fill put in D19 and D20, and a spill never writes over a cell that holds something.
    assert_eq!(value(&s, "D18"), SPILL);
}
