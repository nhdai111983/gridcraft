//! NexusOffice: what a host window that embeds Nexus Sheet in its tabs needs — File › New and
//! Ctrl+N ask the host for a new tab, Open hands a workbook to the host unless this window holds
//! only an untouched blank one, and Open/Save As dialogs answer later without blocking.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};

use egui_kittest::kittest::Queryable as _;
use gridcraft_engine::Session;
use gridcraft_ui_egui::{FileDialogAnswer, FileDialogRequest, Services, SheetApp};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness(services: Services) -> H {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, services);
    app.wps = true;
    app.embedded = true;
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

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nexus-sheet-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// File › New, Ctrl+N and the sample workbooks go to the host (a new tab); this window keeps its
/// one workbook.
#[test]
fn new_workbooks_go_to_the_host() {
    let asked: Rc<RefCell<Vec<serde_json::Value>>> = Rc::default();
    let log = asked.clone();
    let mut h = harness(Services { new_document: Some(Box::new(move |p| log.borrow_mut().push(p))), ..Default::default() });
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::N);
    h.run_steps(3);
    h.state_mut().run_or_alert("file.new", json!({"sample": "sales"}));
    h.run_steps(2);
    assert_eq!(*asked.borrow(), vec![json!({}), json!({"sample": "sales"})]);
    assert_eq!(h.state().session.documents().len(), 1, "no second workbook in this window");
    assert!(h.state().backstage.is_none());
}

/// Without the hook, File › New adds a workbook as upstream does (the stand-alone app).
#[test]
fn without_a_host_new_adds_a_workbook() {
    let mut h = harness(Services::default());
    h.state_mut().run_or_alert("file.new", json!({}));
    h.run_steps(2);
    assert_eq!(h.state().session.documents().len(), 2);
}

/// Open replaces an untouched blank workbook in place; once there is work in the window, the
/// picked workbook goes to the host instead.
#[test]
fn open_replaces_a_blank_workbook_else_goes_to_the_host() {
    let d = dir("open");
    let (a, b) = (d.join("Thu chi.xlsx"), d.join("Công nợ.xlsx"));
    for p in [&a, &b] {
        let mut s = Session::new();
        s.new_workbook();
        s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
        s.execute("file.save", json!({"path": p.to_string_lossy()})).unwrap();
    }
    let opened: Rc<RefCell<Vec<String>>> = Rc::default();
    let log = opened.clone();
    let mut h = harness(Services { open_document: Some(Box::new(move |p| log.borrow_mut().push(p.to_string()))), ..Default::default() });
    h.state_mut().open_path(&a.to_string_lossy());
    h.run_steps(2);
    assert!(opened.borrow().is_empty(), "the blank workbook was replaced in place");
    assert_eq!(h.state().session.active().and_then(|d| d.path.clone()).as_deref(), Some(&*a.to_string_lossy()));
    h.state_mut().open_path(&b.to_string_lossy());
    h.run_steps(2);
    assert_eq!(*opened.borrow(), vec![b.to_string_lossy().into_owned()], "the host opens it in a tab of its own");
    assert_eq!(h.state().session.documents().len(), 1);
    let _ = std::fs::remove_dir_all(&d);
}

/// A host whose dialogs answer later: Save on a new workbook asks, the window keeps drawing, and
/// the workbook is written when the answer comes.
#[test]
fn save_as_waits_for_the_hosts_dialog() {
    let d = dir("save");
    let target = d.join("Bảng lương.xlsx");
    let waiting: Rc<RefCell<Vec<(FileDialogRequest, Sender<FileDialogAnswer>)>>> = Rc::default();
    let log = waiting.clone();
    let mut h = harness(Services {
        file_dialog: Some(Box::new(move |req| {
            let (tx, rx) = channel();
            log.borrow_mut().push((req, tx));
            rx
        })),
        ..Default::default()
    });
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "Lương tháng 10"})).unwrap();
    h.state_mut().run_or_alert("file.save", json!({}));
    h.run_steps(3);
    assert!(h.state().file_dialog_open());
    let (req, tx) = waiting.borrow_mut().pop().unwrap();
    assert!(matches!(&req, FileDialogRequest::Save { name, types } if name.ends_with(".xlsx") && *types == ["xlsx"]), "{req:?}");
    // Another Save while the dialog is up asks nothing more.
    h.state_mut().run_or_alert("file.saveAs", json!({}));
    assert!(waiting.borrow().is_empty());
    tx.send(Ok(Some(d.join("Bảng lương").to_string_lossy().into_owned()))).unwrap();
    h.run_steps(3);
    assert!(target.is_file(), "saved where the dialog said, as .xlsx");
    assert!(!h.state().file_dialog_open());
    assert!(!h.state().session.active().unwrap().is_dirty());
    let _ = std::fs::remove_dir_all(&d);
}

/// "Save changes?" answered Save on a workbook with no file: the workbook closes once the host's
/// Save As has written it; Cancel in the dialog keeps it open.
#[test]
fn closing_a_new_workbook_saves_through_the_hosts_dialog_then_closes() {
    let d = dir("close");
    let waiting: Rc<RefCell<Vec<Sender<FileDialogAnswer>>>> = Rc::default();
    let log = waiting.clone();
    let mut h = harness(Services {
        file_dialog: Some(Box::new(move |_req| {
            let (tx, rx) = channel();
            log.borrow_mut().push(tx);
            rx
        })),
        ..Default::default()
    });
    h.state_mut().ui.language = gridcraft_ui_egui::i18n::Language::Vi;
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "x"})).unwrap();
    let uid = h.state().session.active().unwrap().uid;
    h.state_mut().run_or_alert("file.close", json!({}));
    h.run_steps(3);
    assert!(h.query_by_label("Lưu thay đổi?").is_some() || h.query_by_label_contains("Bạn có muốn lưu").is_some(), "asks in Vietnamese");
    h.get_by_label("Lưu").click();
    h.run_steps(3);
    let tx = waiting.borrow_mut().pop().expect("Save asked where");
    tx.send(Ok(Some(d.join("Sổ quỹ.xlsx").to_string_lossy().into_owned()))).unwrap();
    h.run_steps(3);
    assert!(d.join("Sổ quỹ.xlsx").is_file());
    assert!(h.state().session.documents().iter().all(|x| x.uid != uid), "the saved workbook closed");
    let _ = std::fs::remove_dir_all(&d);
}

/// A cancelled dialog (or a host that went away) changes nothing.
#[test]
fn a_cancelled_dialog_changes_nothing() {
    let mut h = harness(Services {
        file_dialog: Some(Box::new(|_req| {
            let (tx, rx) = channel();
            let _ = tx.send(Ok(None));
            rx
        })),
        ..Default::default()
    });
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "x"})).unwrap();
    h.state_mut().open_dialog("open", json!({}));
    h.state_mut().run_or_alert("file.save", json!({}));
    h.run_steps(3);
    assert!(!h.state().file_dialog_open());
    assert!(h.state().session.active().unwrap().is_dirty());
    assert!(h.state().message.is_none());
}

/// Another editor in the host window (Nexus Docs) installs its own fonts, without this app's
/// families: the embedded app installs them again, keeping the other editor's, instead of
/// panicking on a family egui no longer has. Re-styling on demand works the same way.
#[test]
fn another_editors_fonts_are_kept_and_ours_come_back() {
    let mut h = harness(Services::default());
    let has = |h: &H, name: &str| h.ctx.fonts(|f| f.definitions().families.contains_key(&egui::FontFamily::Name(name.into())));
    assert!(has(&h, "ui"));
    // What the other editor installs: egui's fonts plus its own family.
    let mut other = egui::FontDefinitions::default();
    let chain = other.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    other.families.insert(egui::FontFamily::Name("semibold".into()), chain);
    h.ctx.set_fonts(other);
    h.run_steps(4);
    assert!(has(&h, "ui"), "this app's families are back");
    assert!(has(&h, "semibold"), "the other editor's family stays");
    h.state_mut().restyle();
    h.run_steps(4);
    assert!(has(&h, "ui") && has(&h, "semibold"));
    assert!(h.query_by_label("Tệp").is_some() || h.query_by_label("File").is_some(), "the app draws again");
}

/// Typing a row with Tab between cells and Enter at its end, as people fill a table: every value
/// lands in its cell and the keyboard stays with the grid (Tab never moves egui's focus to a
/// button that Enter would then press).
#[test]
fn tab_and_enter_fill_a_row_and_keep_the_keyboard() {
    for wps in [false, true] {
        tab_and_enter(wps);
    }
}

fn tab_and_enter(wps: bool) {
    let mut h = harness(Services::default());
    h.state_mut().wps = wps;
    h.run_steps(4);
    for row in [["Khoản chi", "Số tiền"], ["Tiền điện", "1250000"]] {
        for (i, cell) in row.iter().enumerate() {
            // (A frame for the cell editor to take the keyboard, as upstream's typing tests do.)
            h.event(egui::Event::Text((*cell).into()));
            h.run_steps(2);
            h.key_press(if i == 0 { egui::Key::Tab } else { egui::Key::Enter });
            h.run_steps(2);
        }
    }
    h.run_steps(2);
    let value = |a1: &str| {
        let at = gridcraft_engine::core::CellRef::parse(a1).unwrap();
        h.state().session.active().and_then(|d| d.wb.active()).map(|sh| sh.value(at).display()).unwrap_or_default()
    };
    let focus = h.ctx.memory(|m| m.focused());
    // Enter after a run of Tabs goes back to the column the run started in (Excel's behaviour).
    assert_eq!([value("A1"), value("B1"), value("A2"), value("B2")], ["Khoản chi", "Số tiền", "Tiền điện", "1250000"], "wps {wps}, focus {focus:?}");
    assert!(h.state().backstage.is_none() && h.state().dialog.is_none(), "no button was pressed");
}

/// A host that brings the tab forward in the middle of a frame draws it before its `logic` runs:
/// with another editor's fonts in the window, `ui` paints nothing that frame (and asks for its
/// fonts again) instead of panicking on a family egui no longer has.
#[test]
fn drawn_before_logic_with_another_editors_fonts_it_waits_a_frame() {
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, Services::default());
    app.wps = true;
    app.embedded = true;
    // (app, run logic before drawing)
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).build_ui_state(
        |ui, s: &mut (SheetApp, bool)| {
            let ctx = ui.ctx().clone();
            if s.1 {
                s.0.logic(&ctx);
            }
            s.0.ui(ui);
        },
        (app, true),
    );
    h.run_steps(6);
    let mut other = egui::FontDefinitions::default();
    let chain = other.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    other.families.insert(egui::FontFamily::Name("semibold".into()), chain);
    h.ctx.set_fonts(other);
    h.state_mut().1 = false;
    h.run_steps(2);
    h.state_mut().1 = true;
    h.run_steps(4);
    let has = h.ctx.fonts(|f| f.definitions().families.contains_key(&egui::FontFamily::Name("ui".into())));
    assert!(has, "its fonts came back");
}

/// In Vietnamese, new sheets are "Trang tính n" and the accounting format is đồng's.
#[test]
fn vietnamese_names_sheets_and_formats_dong() {
    let mut h = harness(Services::default());
    h.state_mut().ui.language = gridcraft_ui_egui::i18n::Language::Vi;
    h.run_steps(4);
    h.state_mut().run_or_alert("home.insertSheet", json!({}));
    h.run_steps(2);
    let names: Vec<String> = h.state().session.active().unwrap().wb.sheets.iter().map(|s| s.name.clone()).collect();
    assert_eq!(names.last().map(String::as_str), Some("Trang tính1"), "{names:?}");
    h.state_mut().run("cell.set", json!({"cell": "A1", "input": "1250000"})).unwrap();
    h.get_by_label("Định dạng số kế toán").click();
    h.run_steps(3);
    h.get_by_label("₫ Tiếng Việt").click();
    h.run_steps(3);
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    let at = gridcraft_engine::core::CellRef::parse("A1").unwrap();
    let code = d.wb.styles.get(sh.style_id(at)).num_fmt.as_str().to_string();
    assert!(code.contains('₫'), "{code}");
    let shown = gridcraft_engine::numfmt::format_value(&sh.value(at), &gridcraft_engine::numfmt::NumberFormat::parse(&code), d.wb.date_system).text;
    assert!(shown.contains("1,250,000") && shown.contains('₫'), "{shown:?}");
}
