//! Nexus Sheet: COUNTIF tests of fix/text-compare-canonical (M3-SCOLL.2), which upstream keeps in
//! `crates/functions/src/math.rs`. They live here so the port does not modify math.rs (patch budget);
//! they use only the crate's public API (`lookup` + `call`).

use gridcraft_core::{Array, DateSystem, Value};
use gridcraft_functions::{Arg, Ctx, call, lookup};

struct TestCtx;

impl Ctx for TestCtx {
    fn date_system(&self) -> DateSystem {
        DateSystem::D1900
    }
    fn now_serial(&self) -> f64 {
        46302.5
    }
    fn random(&mut self) -> f64 {
        0.5
    }
}

fn ev(name: &str, args: Vec<Arg>) -> Value {
    let spec = lookup(name).unwrap_or_else(|| panic!("function {name} not registered"));
    call(spec, &args, &mut TestCtx)
}

fn col_t(xs: &[&str]) -> Value {
    let cells: Vec<Value> = xs.iter().map(|x| Value::from(*x)).collect();
    Value::from(Array::new(xs.len(), 1, cells).expect("rectangular"))
}

#[track_caller]
fn close(v: Value, expected: f64) {
    match v {
        Value::Number(x) => assert!((x - expected).abs() < 1e-9, "got {x}, expected {expected}"),
        other => panic!("expected {expected}, got {other:?}"),
    }
}

const NFC: &str = "Nguy\u{1ec5}n Th\u{1ecb} Ng\u{1ecd}c \u{c1}nh";
const NFD: &str = "Nguye\u{302}\u{303}n Thi\u{323} Ngo\u{323}c A\u{301}nh";

#[test]
fn countif_matches_nfc_and_nfd() {
    let v = col_t(&[NFC, NFD]);
    close(ev("COUNTIF", vec![Arg::reference(v.clone()), Arg::val(NFC)]), 2.0);
    close(ev("COUNTIF", vec![Arg::reference(v), Arg::val(NFD)]), 2.0);
    // Not-equal and wildcard patterns: `<>` is the complement, wildcards keep their per-char matching.
    let w = col_t(&[NFC, NFD, "x"]);
    close(ev("COUNTIF", vec![Arg::reference(w.clone()), Arg::val(format!("<>{NFD}"))]), 1.0);
    close(ev("COUNTIF", vec![Arg::reference(w), Arg::val("Nguy*")]), 2.0);
}

#[test]
fn relational_criteria_follow_root_order() {
    let v = col_t(&["\u{1ea5}m", "b\u{e0}n", "Zebra"]);
    close(ev("COUNTIF", vec![Arg::reference(v), Arg::val("<b")]), 1.0);
}
