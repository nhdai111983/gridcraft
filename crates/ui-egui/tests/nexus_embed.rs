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
    assert!(matches!(&req, FileDialogRequest::Save { name } if name.ends_with(".xlsx")), "{req:?}");
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
