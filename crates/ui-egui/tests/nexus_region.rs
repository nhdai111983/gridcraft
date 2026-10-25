//! NexusOffice: the regional format apart from the interface language (File › Options › Region,
//! `ui.region`), and format codes in Format Cells shown in it while the file keeps en-US codes.

use egui_kittest::kittest::Queryable;
use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Locale, Value};
use gridcraft_ui_egui::{SheetApp, i18n::Language};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness(language: Language) -> H {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, Default::default());
    app.ui.language = language;
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

fn shown(h: &H, a1: &str) -> String {
    let d = h.state().session.active().unwrap();
    gridcraft_engine::display::cell_text(&d.wb, d.wb.active().unwrap(), CellRef::parse(a1).unwrap())
}

fn code(h: &H, a1: &str) -> String {
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(CellRef::parse(a1).unwrap())).num_fmt.as_str().to_string()
}

fn enter(h: &mut H, a1: &str, typed: &str) {
    h.state_mut().run("selection.set", json!({"cell": a1})).unwrap();
    h.state_mut().begin_edit(Some(typed.into()), false);
    assert!(h.state_mut().commit_edit(0, 0, false, false), "{typed}");
}

/// English menus with the vi-VN region: numbers, dates and formulas the vi-VN way. Clearing the
/// region goes back to the language's; the setting is saved with the preferences.
#[test]
fn region_is_set_apart_from_the_language() {
    let mut h = harness(Language::En);
    assert_eq!(h.state().session.locale, Locale::EnUs);
    let r = h.state_mut().run("ui.region", json!({"value": "vi-VN"})).unwrap();
    assert_eq!(r, json!({"region": "vi-VN", "effective": "vi-VN"}));
    h.run_steps(2);
    assert_eq!(h.state().session.locale, Locale::ViVn);
    assert_eq!(h.state().ui.language, Language::En, "the menus stay English");
    enter(&mut h, "A1", "1.234,5");
    enter(&mut h, "A2", "25/10/2026");
    enter(&mut h, "A3", "=SUM(A1;0,5)");
    enter(&mut h, "A4", "=COUNTIF(A1:A3;\">1,5\")");
    assert_eq!(value(&h, "A1"), Value::Number(1234.5));
    assert_eq!(value(&h, "A3"), Value::Number(1235.0));
    assert_eq!(value(&h, "A4"), Value::Number(3.0));
    assert_eq!(shown(&h, "A1"), "1.234,50");
    assert_eq!(shown(&h, "A2"), "25/10/2026");
    let saved = serde_json::to_value(&h.state().ui).unwrap();
    assert_eq!(saved["region"], json!("vi-VN"));
    // Vietnamese menus with the en-US region.
    h.state_mut().run("app.language.set", json!({"language": "vi"})).unwrap();
    h.state_mut().run("ui.region", json!({"value": "en-US"})).unwrap();
    h.run_steps(2);
    assert_eq!(h.state().session.locale, Locale::EnUs);
    assert_eq!(shown(&h, "A1"), "1,234.50");
    enter(&mut h, "B1", "1.5");
    enter(&mut h, "B2", "=SUM(B1,1)");
    assert_eq!(value(&h, "B1"), Value::Number(1.5));
    assert_eq!(value(&h, "B2"), Value::Number(2.5));
    // Cleared, it follows the language again (Vietnamese → vi-VN).
    h.state_mut().run("ui.region", json!({"value": ""})).unwrap();
    h.run_steps(2);
    assert_eq!(h.state().session.locale, Locale::ViVn);
    assert_eq!(serde_json::to_value(&h.state().ui).unwrap().get("region"), None);
    // The control channel stays en-US whatever the region (PR #13's rule).
    h.state_mut().run("ui.region", json!({"value": "vi-VN"})).unwrap();
    h.run_steps(2);
    h.state_mut().session.execute("cell.set", json!({"cell": "C1", "input": "1.5"})).unwrap();
    assert_eq!(value(&h, "C1"), Value::Number(1.5));
    // Junk never panics and changes nothing.
    for junk in [json!("de-DE"), json!(1), json!({"x": "vi-VN"})] {
        assert!(h.state_mut().run("ui.region", json!({"value": junk})).is_err());
    }
    assert_eq!(h.state().ui.region, Some(Locale::ViVn));
}

/// File › Options shows the Region row; picking a region applies it.
#[test]
fn options_page_picks_the_region() {
    let mut h = harness(Language::Vi);
    h.state_mut().run("ui.backstage", json!({"page": "options"})).unwrap();
    h.run_steps(6);
    h.get_by_label("Vùng (số và ngày)");
    h.get_by_value("Theo ngôn ngữ giao diện — Tiếng Việt (Việt Nam)").click();
    h.run_steps(4);
    h.get_by_label("English (United States)").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.region, Some(Locale::EnUs));
    assert_eq!(h.state().session.locale, Locale::EnUs);
}

fn open_number_tab(h: &mut H) {
    h.state_mut().open_dialog("formatCells", json!({"tab": "Number"}));
    h.run_steps(6);
}

fn ok(h: &mut H) {
    h.get_by_label("   OK   ").click();
    h.run_steps(6);
    assert!(h.state().dialog.is_none());
}

/// Format Cells in vi-VN shows codes the vi-VN way (`#.##0,00`, `#.##0 ₫`'s code, the system date
/// as `dd/mm/yyyy`) and stores the en-US code. Opening and confirming changes nothing.
#[test]
fn format_cells_codes_in_the_regional_format() {
    let mut h = harness(Language::En);
    h.state_mut().run("ui.region", json!({"value": "vi-VN"})).unwrap();
    h.run_steps(2);
    h.state_mut().session.execute("cell.set", json!({"cell": "A1", "input": "1234.5"})).unwrap();
    h.state_mut().session.execute("home.numberFormat", json!({"range": "A1", "code": "#,##0.00"})).unwrap();
    h.state_mut().session.execute("cell.set", json!({"cell": "A2", "input": "10/25/2026"})).unwrap();
    h.state_mut().run("selection.set", json!({"cell": "A1"})).unwrap();
    open_number_tab(&mut h);
    // The code field and the presets are spelled vi-VN; nothing is spelled en-US.
    assert!(h.query_by_value("#,##0.00").is_none() && h.query_all_by_value("#.##0,00").next().is_some(), "the code field");
    h.get_by_label("#.##0,00");
    h.get_by_label("0,00%");
    assert!(h.query_by_label("#,##0.00").is_none());
    ok(&mut h);
    assert_eq!(code(&h, "A1"), "#,##0.00", "unchanged by opening the dialog");
    // A preset is stored en-US.
    open_number_tab(&mut h);
    h.get_by_label("#.##0").click();
    h.run_steps(2);
    ok(&mut h);
    assert_eq!(code(&h, "A1"), "#,##0");
    assert_eq!(shown(&h, "A1"), "1.235");
    // Currency is đồng's in vi-VN.
    open_number_tab(&mut h);
    h.get_by_label("Currency").click();
    h.run_steps(2);
    assert!(h.query_all_by_value("#.##0 [$₫-42A]").next().is_some());
    ok(&mut h);
    assert_eq!(code(&h, "A1"), "#,##0 [$₫-42A]");
    // The system short date shows day first and stays the system date when confirmed.
    h.state_mut().run("selection.set", json!({"cell": "A2"})).unwrap();
    assert_eq!(code(&h, "A2"), "m/d/yyyy");
    open_number_tab(&mut h);
    assert!(h.query_all_by_value("dd/mm/yyyy").next().is_some() && h.query_by_value("m/d/yyyy").is_none());
    ok(&mut h);
    assert_eq!(code(&h, "A2"), "m/d/yyyy");
    // A typed code reads vi-VN.
    h.state_mut().run("selection.set", json!({"cell": "A1"})).unwrap();
    open_number_tab(&mut h);
    h.get_all_by_value("#.##0 [$₫-42A]").next().unwrap().click();
    h.run_steps(2);
    select_all_and_type(&mut h, "0,000");
    assert!(h.query_all_by_value("0,000").next().is_some());
    ok(&mut h);
    assert_eq!(code(&h, "A1"), "0.000");
    // The XLSX written keeps the en-US codes.
    let bytes = h.state_mut().session.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let mut s = Session::new();
    s.execute("file.open", json!({"base64": bytes["base64"], "name": "t.xlsx"})).unwrap();
    let d = s.active().unwrap();
    let sh = d.wb.active().unwrap();
    assert_eq!(d.wb.styles.get(sh.style_id(CellRef::parse("A1").unwrap())).num_fmt.as_str(), "0.000");
    assert_eq!(d.wb.styles.get(sh.style_id(CellRef::parse("A2").unwrap())).num_fmt.as_str(), "m/d/yyyy");
}

fn select_all_and_type(h: &mut H, text: &str) {
    let m = if cfg!(target_os = "macos") { egui::Modifiers::MAC_CMD } else { egui::Modifiers::CTRL };
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(egui::Event::Key { key: egui::Key::A, physical_key: None, pressed: false, repeat: false, modifiers: m });
    h.run_steps(1);
    h.input_mut().events.push(egui::Event::Text(text.into()));
    h.run_steps(2);
}

/// en-US Format Cells is unchanged: codes as stored.
#[test]
fn format_cells_codes_in_en_us_are_unchanged() {
    let mut h = harness(Language::En);
    h.state_mut().session.execute("cell.set", json!({"cell": "A1", "input": "1234.5"})).unwrap();
    h.state_mut().session.execute("home.numberFormat", json!({"range": "A1", "code": "#,##0.00"})).unwrap();
    h.state_mut().run("selection.set", json!({"cell": "A1"})).unwrap();
    open_number_tab(&mut h);
    assert!(h.query_all_by_value("#,##0.00").next().is_some());
    h.get_by_label("0.00%").click();
    h.run_steps(2);
    ok(&mut h);
    assert_eq!(code(&h, "A1"), "0.00%");
}
