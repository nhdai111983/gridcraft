//! NexusOffice: typing in the grid — Vietnamese numbers, dates and đồng through the cell editor
//! (vi-VN regional format), a Tab pressed the frame after a character, and Excel's Enter after a
//! run of Tabs.

use egui::{Event, Key, Modifiers};
use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Locale, Value};
use gridcraft_ui_egui::{SheetApp, i18n::Language};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness(language: Language, wps: bool) -> H {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, Default::default());
    app.ui.language = language;
    app.wps = wps;
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(6);
    h
}

fn value(h: &H, a1: &str) -> Value {
    h.state().session.active().and_then(|d| d.wb.active().map(|s| s.value(CellRef::parse(a1).unwrap()))).unwrap_or_default()
}

fn active(h: &H) -> String {
    h.state().session.active().unwrap().selection.active.a1()
}

/// One frame per event, as fast typing delivers them.
fn text_now(h: &mut H, t: &str) {
    h.input_mut().events.push(Event::Text(t.into()));
    h.run_steps(1);
}

fn key_now(h: &mut H, k: Key, m: Modifiers) {
    h.input_mut().events.push(Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(Event::Key { key: k, physical_key: None, pressed: false, repeat: false, modifiers: m });
    h.run_steps(1);
}

/// A character, Tab on the very next frame, then the next character: every key reaches the grid
/// (the Tab used to send egui's focus to the next widget, and `b` was lost).
#[test]
fn tab_right_after_a_character_keeps_the_keyboard() {
    for wps in [false, true] {
        let mut h = harness(Language::En, wps);
        text_now(&mut h, "a");
        key_now(&mut h, Key::Tab, Modifiers::NONE);
        text_now(&mut h, "b");
        key_now(&mut h, Key::Tab, Modifiers::NONE);
        text_now(&mut h, "c");
        key_now(&mut h, Key::Enter, Modifiers::NONE);
        text_now(&mut h, "d");
        key_now(&mut h, Key::Enter, Modifiers::NONE);
        h.run_steps(2);
        assert_eq!(
            [value(&h, "A1"), value(&h, "B1"), value(&h, "C1"), value(&h, "A2")],
            [Value::text("a"), Value::text("b"), Value::text("c"), Value::text("d")],
            "wps {wps}, focus {:?}",
            h.ctx.memory(|m| m.focused())
        );
        assert!(h.state().dialog.is_none() && h.state().backstage.is_none(), "no button was pressed");
    }
}

/// Excel: after Tab entries along a row, Enter goes to the next row at the column the run
/// started in — also when the run started mid-row and without editing; an arrow key ends the run.
#[test]
fn enter_after_tabs_returns_to_the_runs_first_column() {
    let mut h = harness(Language::En, false);
    h.state_mut().run("selection.set", json!({"cell": "C3"})).unwrap();
    h.run_steps(2);
    for t in ["x", "y", "z"] {
        text_now(&mut h, t);
        key_now(&mut h, Key::Tab, Modifiers::NONE);
    }
    assert_eq!(active(&h), "F3");
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(active(&h), "C4");
    // Tab without typing is part of a run too; Shift+Tab inside it keeps it.
    key_now(&mut h, Key::Tab, Modifiers::NONE);
    key_now(&mut h, Key::Tab, Modifiers::NONE);
    key_now(&mut h, Key::Tab, Modifiers::SHIFT);
    assert_eq!(active(&h), "D4");
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(active(&h), "C5");
    // Enter alone goes straight down.
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(active(&h), "C6");
    // An arrow ends the run: Enter then goes straight down.
    key_now(&mut h, Key::Tab, Modifiers::NONE);
    key_now(&mut h, Key::ArrowRight, Modifiers::NONE);
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(active(&h), "E7");
    // So does an arrow that commits a typed entry.
    key_now(&mut h, Key::Tab, Modifiers::NONE);
    text_now(&mut h, "q");
    key_now(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!(active(&h), "F8");
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(active(&h), "F9");
}

fn enter(h: &mut H, a1: &str, typed: &str) {
    h.state_mut().run("selection.set", json!({"cell": a1})).unwrap();
    h.state_mut().begin_edit(Some(typed.into()), false);
    assert!(h.state_mut().commit_edit(0, 0, false, false), "{typed}");
}

/// In Vietnamese, values are typed and shown the vi-VN way; formulas take `;` and decimal commas
/// with English function names; the file and the API stay en-US.
#[test]
fn vietnamese_entry_and_display() {
    let mut h = harness(Language::Vi, false);
    assert_eq!(h.state().session.locale, Locale::ViVn, "the locale follows the interface language");
    assert_eq!(h.state().session.active().unwrap().wb.locale, Locale::ViVn);
    enter(&mut h, "A1", "1.234,5");
    enter(&mut h, "A2", "24/10/2026");
    enter(&mut h, "A3", "1.234.567 ₫");
    enter(&mut h, "A4", "=SUM(A1;0,5)");
    enter(&mut h, "A5", "Thứ Hai");
    enter(&mut h, "A6", "1,234.5");
    // NFD text typed (macOS, some IMEs) is kept as typed.
    enter(&mut h, "A7", "Vie\u{302}\u{323}t");
    assert_eq!(value(&h, "A1"), Value::Number(1234.5));
    assert_eq!(value(&h, "A2"), Value::Number(46319.0));
    assert_eq!(value(&h, "A3"), Value::Number(1_234_567.0));
    assert_eq!(value(&h, "A4"), Value::Number(1235.0));
    assert_eq!(value(&h, "A5"), Value::text("Thứ Hai"));
    assert_eq!(value(&h, "A6"), Value::text("1,234.5"));
    assert_eq!(value(&h, "A7"), Value::text("Vie\u{302}\u{323}t"));
    let formula = h.state().session.active().unwrap().wb.active().unwrap().cell(CellRef::parse("A4").unwrap()).unwrap().formula.clone().unwrap();
    assert_eq!(formula.text.to_string(), "SUM(A1,0.5)", "stored en-US");
    let shown = |h: &H, a1: &str| {
        let d = h.state().session.active().unwrap();
        gridcraft_engine::display::cell_text(&d.wb, d.wb.active().unwrap(), CellRef::parse(a1).unwrap())
    };
    assert_eq!(shown(&h, "A1"), "1.234,50");
    assert_eq!(shown(&h, "A2"), "24/10/2026");
    assert_eq!(shown(&h, "A3"), "1.234.567 ₫");
    // Editing shows the vi-VN spelling and commits it unchanged.
    for (a1, edit) in [("A1", "1234,5"), ("A2", "24/10/2026"), ("A4", "=SUM(A1;0,5)")] {
        h.state_mut().run("selection.set", json!({"cell": a1})).unwrap();
        h.state_mut().begin_edit(None, false);
        assert_eq!(h.state().editor.as_ref().unwrap().text, edit, "{a1}");
        assert!(h.state_mut().commit_edit(0, 0, false, false));
    }
    assert_eq!(value(&h, "A1"), Value::Number(1234.5));
    assert_eq!(value(&h, "A2"), Value::Number(46319.0));
    // Switching to English shows the same values the en-US way.
    h.state_mut().run("app.language.set", json!({"language": "en"})).unwrap();
    h.run_steps(2);
    assert_eq!(h.state().session.active().unwrap().wb.locale, Locale::EnUs);
    assert_eq!(shown(&h, "A1"), "1,234.50");
    assert_eq!(shown(&h, "A2"), "10/24/2026");
    assert_eq!(shown(&h, "A3"), "1,234,567 ₫");
}

/// The cell editor reads a Vietnamese date typed key by key the same way.
#[test]
fn vietnamese_values_typed_key_by_key() {
    let mut h = harness(Language::Vi, false);
    for t in ["1", ".", "2", "3", "4", ",", "5"] {
        text_now(&mut h, t);
    }
    key_now(&mut h, Key::Tab, Modifiers::NONE);
    for t in "24/10/2026".chars() {
        text_now(&mut h, &t.to_string());
    }
    key_now(&mut h, Key::Enter, Modifiers::NONE);
    h.run_steps(2);
    assert_eq!(value(&h, "A1"), Value::Number(1234.5));
    assert_eq!(value(&h, "B1"), Value::Number(46319.0));
    assert_eq!(active(&h), "A2");
}
