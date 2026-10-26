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
];

/// An English label: ASCII only, with a word of three letters or more (not all capitals, so
/// function names, cell references and format codes pass) that is not in [`KEEP`].
fn english(label: &str) -> bool {
    if !label.is_ascii() {
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
        json!({"message": "Solver found a solution. All Constraints and optimality conditions are satisfied.", "changed": true, "values": {"B2": 1}}),
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
        assert!(added.iter().any(|l| l.as_str() == want), "{want} missing: {added:?}");
    }
    let bad: Vec<&&String> = added.iter().filter(|l| english(l)).collect();
    assert!(bad.is_empty(), "English in the sheet tab menu: {bad:?}");
}
