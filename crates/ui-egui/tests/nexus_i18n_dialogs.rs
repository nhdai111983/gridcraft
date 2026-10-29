//! NexusOffice (QC-SHT-02, -03, -06): with the interface in Vietnamese, every dialog, the sheet-tab
//! menu and the status bar show Vietnamese. Upstream draws many labels as English literals, so a
//! translation in `i18n/vi.rs` never reached the screen; `nexus_i18n::t` translates them where
//! they are drawn. This test opens each dialog in Vietnamese and fails on any label it adds that
//! is still English.

use std::collections::BTreeSet;

use egui::accesskit::Role;
use egui_kittest::kittest::{By, NodeT, Queryable};
use gridcraft_engine::Session;
use gridcraft_ui_egui::{SheetApp, i18n::Language};
use serde_json::{Value as Json, json};

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness() -> H {
    let mut session = Session::new();
    session.new_workbook();
    // Vietnamese data only, so a label taken from the sheet is never mistaken for English.
    session
        .execute(
            "range.setValues",
            json!({"range": "A1", "values": [["Họ tên", "Số tiền", "Ngày"], ["Đặng Văn Ân", 1234.5, 46321], ["Bùi Thị Ánh", 2000, 46322], ["Ân Đức", 30, 46323], ["Anh Tuấn", 45.25, 46324]]}),
        )
        .unwrap();
    session.execute("selection.set", json!({"range": "A1:C5"})).unwrap();
    let mut app = SheetApp::new(session, Default::default());
    app.ui.language = Language::Vi;
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1400.0, 900.0)).with_max_steps(16).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(4);
    h
}

/// Every label and value on screen.
fn labels(h: &H) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for n in h.query_all(By::new().include_labels().predicate(|_| true)) {
        let node = n.accesskit_node();
        for text in [node.label(), node.value()].into_iter().flatten() {
            for line in text.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    out.insert(line.to_string());
                }
            }
        }
    }
    out
}

/// Words that may stay as they are in a Vietnamese interface: product and format names, and the
/// Vietnamese words written without diacritics.
const KEEP: &[&str] = &[
    "Nexus",
    "Sheet",
    "NexusOffice",
    "SonnyTech",
    "Solver",
    "Simplex",
    "GRG",
    "Anova",
    "Pareto",
    "Kth",
    "nexus",
    "sheet",
    "cli",
    "mcp",
    "control",
    "connect",
    "headless",
    "server",
    "channel",
    "lines",
    "JSON",
    "docs",
    "md",
    "protocol",
    "png",
    "jpeg",
    "xlsx",
    "csv",
    "html",
    "Tab",
    "Trang",
    "tinh",
    "Excel",
    "Fisher",
    "VBA",
    "Unicode",
    "Latin",
    "Arial",
    "Calibri",
    "Aptos",
    "Liberation",
    "Sans",
    "Serif",
    "Noto",
    // Same word in Vietnamese (Excel's Vietnamese edition writes these as they are).
    "Chia",
    "Macro",
    "Alpha",
    "Web",
];

/// Accessibility names egui itself gives its widgets (the window close button), which it offers
/// no way to translate. Screen readers only; nothing on screen.
const EGUI_OWN: &[&str] = &["Close window"];

/// A number format code (`dd/mm/yyyy`, `#.##0;[Red]-#.##0`, `General`): codes are the same in
/// every language.
fn format_code(label: &str) -> bool {
    let mut rest = String::new();
    let mut depth = 0;
    for c in label.chars() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ if depth == 0 => rest.push(c),
            _ => {}
        }
    }
    let words: Vec<&str> = rest.split(|c: char| !c.is_ascii_alphabetic()).filter(|w| !w.is_empty()).collect();
    (!words.is_empty() || label.contains(['#', '0']))
        && words.iter().all(|w| *w == "General" || *w == "AM" || *w == "PM" || w.chars().all(|c| "dmyhsDMYHSE".contains(c)))
}

/// An English label: ASCII only, with a word of three letters or more (not all capitals, so
/// function names, cell references and format codes pass) that is not in [`KEEP`].
fn english(label: &str) -> bool {
    if !label.is_ascii() || EGUI_OWN.contains(&label) || label.starts_with("http") || format_code(label) {
        return false;
    }
    label.split(|c: char| !c.is_ascii_alphabetic()).any(|w| w.len() >= 3 && w.chars().any(|c| c.is_ascii_lowercase()) && !KEEP.contains(&w))
}

fn close(h: &mut H) {
    h.state_mut().dialog = None;
    h.state_mut().message = None;
    h.run_steps(2);
}

/// Opens `name` (with `params`) and returns the English labels it added to the screen.
fn untranslated(h: &mut H, base: &BTreeSet<String>, name: &str, params: Json) -> Vec<String> {
    h.state_mut().open_dialog(name, params);
    h.run_steps(4);
    let shown = labels(h);
    let bad: Vec<String> = shown.difference(base).filter(|l| english(l)).cloned().collect();
    close(h);
    bad
}

/// Every dialog the interface opens by name (`Dialog::open`), with the parameters a menu passes.
fn dialogs() -> Vec<(&'static str, Json)> {
    let mut v: Vec<(&'static str, Json)> = [
        "insertCells",
        "insertPaste",
        "deleteCells",
        "renameSheet",
        "moveSheet",
        "unhideSheet",
        "protectSheet",
        "unprotectSheet",
        "allowEditRanges",
        "protectWorkbook",
        "unprotectWorkbook",
        "openPassword",
        "columnWidth",
        "rowHeight",
        "defaultWidth",
        "dataValidation",
        "textToColumns",
        "removeDuplicates",
        "subtotal",
        "zoom",
        "cfQuick",
        "series",
        "defineName",
        "insertFunction",
        "find",
        "goTo",
        "goToSpecial",
        "sort",
        "nameManager",
        "manageRules",
        "recommendedCharts",
        "commandSearch",
        "goalSeek",
        "analysisToolPak",
        "solver",
        "pageSetup",
        "headerFooter",
        "forecastSheet",
        "insertLink",
        "comment",
        "pickList",
        "watch",
        "pasteSpecial",
        "sparkline",
        "pivotCalculatedField",
        "saveCopy",
        "formatCells",
        "macros",
        "insertPicture",
    ]
    .into_iter()
    .map(|n| (n, json!({})))
    .collect();
    v.push(("find", json!({"tab": "Replace"})));
    v.push((
        "solverResults",
        json!({"message": "Solver found an optimal solution. All constraints are satisfied.", "changed": true, "values": {"B2": 1}}),
    ));
    v.push(("openLink", json!({"url": "https://example.com"})));
    for (id, _) in gridcraft_engine::cmd::analysis::TOOLS {
        v.push(("analysisToolPak", json!({"tool": id})));
    }
    v
}

#[test]
fn every_dialog_shows_vietnamese() {
    let mut h = harness();
    let base = labels(&h);
    let mut failures = Vec::new();
    for (name, params) in dialogs() {
        let bad = untranslated(&mut h, &base, name, params.clone());
        if !bad.is_empty() {
            failures.push(format!("{name} {params}: {bad:?}"));
        }
    }
    assert!(failures.is_empty(), "English left in Vietnamese dialogs:\n{}", failures.join("\n"));
}

/// The detector itself: English is caught, Vietnamese and names pass.
#[test]
fn english_detector() {
    assert!(english("My list has headers"));
    assert!(english("Sort by"));
    assert!(english("Average: 500.494,8125"));
    assert!(!english("Danh sách có hàng tiêu đề"));
    assert!(!english("SUM"));
    assert!(!english("A1:C5"));
    assert!(!english("Nexus Sheet"));
    assert!(!english("#,##0.00"));
    assert!(!english("mmmm d, yyyy"));
    assert!(!english("#.##0;[Red]-#.##0"));
    assert!(english("Generally fine"));
}

/// QC-SHT-02: Sum, Count and Average in the status bar.
#[test]
fn status_bar_is_vietnamese() {
    let mut h = harness();
    h.state_mut().session.execute("selection.set", json!({"range": "B2:B5"})).unwrap();
    h.run_steps(3);
    let shown = labels(&h);
    let find = |prefix: &str| shown.iter().find(|l| l.starts_with(prefix)).cloned();
    assert!(find("Tổng: ").is_some(), "{shown:?}");
    assert!(find("Số lượng: ").is_some(), "{shown:?}");
    assert!(find("Trung bình: ").is_some(), "{shown:?}");
    assert!(shown.iter().all(|l| !l.starts_with("Sum:") && !l.starts_with("Count:") && !l.starts_with("Average:")), "{shown:?}");
}

/// QC-SHT-03: the sheet tab's context menu.
#[test]
fn sheet_tab_menu_is_vietnamese() {
    let mut h = harness();
    let base = labels(&h);
    let tab = h.state().session.active().unwrap().wb.sheets[0].name.clone();
    h.get_all_by_label(&tab).find(|n| n.accesskit_node().role() != Role::TextInput).unwrap().click_secondary();
    h.run_steps(4);
    let shown = labels(&h);
    let added: Vec<&String> = shown.difference(&base).collect();
    for want in ["Chèn trang tính", "Đổi tên", "Màu thẻ", "Ẩn", "Chọn tất cả trang tính"] {
        // A submenu's label carries its arrow ("Màu thẻ ⏵").
        assert!(added.iter().any(|l| l.as_str() == want || l.strip_prefix(want).is_some_and(|r| r.trim() == "⏵")), "{want} missing: {added:?}");
    }
    let bad: Vec<&&String> = added.iter().filter(|l| english(l)).collect();
    assert!(bad.is_empty(), "English in the sheet tab menu: {bad:?}");
}

fn click_at(h: &mut H, pos: egui::Pos2, button: egui::PointerButton) {
    h.input_mut().events.push(egui::Event::PointerMoved(pos));
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: true, modifiers: egui::Modifiers::NONE });
    h.input_mut().events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: egui::Modifiers::NONE });
    h.run_steps(3);
}

/// QC-SHT-11: right-clicking a chart opens the chart's menu (in Vietnamese), not the cell menu.
#[test]
fn right_click_on_a_chart_opens_the_chart_menu() {
    let mut h = harness();
    h.state_mut().session.execute("selection.set", json!({"range": "A1:B5"})).unwrap();
    h.state_mut().session.execute("insert.chart", json!({"type": "column"})).unwrap();
    h.state_mut().selected_chart = None;
    assert!(h.state().dialog.is_none());
    h.run_steps(4);
    // A point on the chart.
    let at = {
        let app = h.state();
        let sh = app.session.active().unwrap().wb.active().unwrap();
        let geo = gridcraft_ui_egui::grid::Geo::new(sh, app.grid.rect.unwrap(), app.view().scroll);
        let r = geo.rect;
        let mut found = None;
        'scan: for i in 1..40 {
            for j in 1..40 {
                let p = egui::pos2(r.left() + r.width() * i as f32 / 40.0, r.top() + r.height() * j as f32 / 40.0);
                if gridcraft_ui_egui::chartview::hit(app, &geo, sh, p).is_some_and(|(kind, ..)| kind == "chart") {
                    found = Some(p);
                    break 'scan;
                }
            }
        }
        found.expect("the chart is on screen")
    };
    let base = labels(&h);
    click_at(&mut h, at, egui::PointerButton::Secondary);
    let shown = labels(&h);
    let added: Vec<&String> = shown.difference(&base).collect();
    for want in ["Đổi loại biểu đồ…", "Chọn dữ liệu…", "Xóa"] {
        assert!(
            added.iter().any(|l| l.as_str() == want),
            "{want} missing: {added:?}; at {at:?}; cell menu {:?}; chart {:?}",
            h.state().grid.context_menu,
            h.state().selected_chart
        );
    }
    assert!(h.state().grid.context_menu.is_none(), "not the cell menu");
    assert!(h.state().selected_chart.is_some(), "the chart is selected");
    // Delete from the menu removes the chart.
    h.get_by_label("Xóa").click();
    h.run_steps(3);
    assert!(h.state().session.active().unwrap().wb.active().unwrap().charts.is_empty());
}

/// QC-SHT-13: editing a long entry in A2 widens the editor a whole column at a time, so the end
/// of B2's number never shows right after the text being edited.
#[test]
fn cell_editor_covers_whole_neighbour_columns() {
    let mut h = harness();
    h.state_mut().session.execute("selection.set", json!({"cell": "A2"})).unwrap();
    h.state_mut().begin_edit(None, false);
    h.run_steps(3);
    let editor = h
        .query_all(By::new().predicate(|n| matches!(n.role(), Role::TextInput | Role::MultilineTextInput)))
        .filter(|n| n.accesskit_node().value().is_some_and(|v| v == "Đặng Văn Ân"))
        .map(|n| n.rect())
        // The formula bar shows the same text; the cell editor is the narrow one.
        .min_by(|a, b| a.width().total_cmp(&b.width()))
        .expect("the cell editor");
    let app = h.state();
    let sh = app.session.active().unwrap().wb.active().unwrap();
    let geo = gridcraft_ui_egui::grid::Geo::new(sh, app.grid.rect.unwrap(), app.view().scroll);
    let a2 = geo.cell_rect(sh, gridcraft_engine::core::CellRef::parse("A2").unwrap());
    assert!(editor.right() > a2.right(), "the text is wider than A: {editor:?} vs {a2:?}");
    // The editor (inset 2 px in its box) ends on a column edge.
    let edge = (1..6).map(|c| geo.x(sh, c)).min_by(|a, b| (a - editor.right()).abs().total_cmp(&(b - editor.right()).abs())).unwrap();
    assert!((editor.right() + 2.0 - edge).abs() <= 1.5, "editor ends at {} not on a column edge ({edge})", editor.right());
}
