//! NexusOffice: "Lưu thay đổi?" as a tab of the NexusOffice window — the prompt every Nexus app
//! shows (`nexus_save_prompt`): the title, the question naming the workbook, Lưu · Không lưu · Hủy
//! right-aligned in that order, the keyboard on Lưu, Enter saves and Escape cancels.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use gridcraft_engine::Session;
use gridcraft_ui_egui::i18n::Language;
use gridcraft_ui_egui::nexus_save_prompt::{BUTTON_SIZE, PADDING, WIDTH};
use gridcraft_ui_egui::{Services, SheetApp};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

const NAME: &str = "Báo cáo.xlsx";

/// An embedded Vietnamese window with `NAME` open and changed, asking "Lưu thay đổi?".
fn asking(tag: &str, embedded: bool) -> (H, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("nexus-sheet-save-prompt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(NAME);
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    s.execute("file.save", json!({"path": path.to_string_lossy()})).unwrap();
    let mut session = Session::new();
    session.new_workbook();
    let mut app = SheetApp::new(session, Services::default());
    app.wps = true;
    app.embedded = embedded;
    app.ui.language = Language::Vi;
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        app,
    );
    h.run_steps(6);
    h.state_mut().open_path(&path.to_string_lossy());
    h.run_steps(3);
    h.state_mut().run_or_alert("cell.set", json!({"cell": "A1", "input": "2"}));
    h.state_mut().run_or_alert("file.close", json!({}));
    h.run_steps(6);
    assert_eq!(h.state().dialog.as_ref().map(|d| d.name.as_str()), Some("saveChanges"));
    (h, path)
}

/// The prompt's button `label` (the tool row above may have a button of the same name).
fn button(h: &H, label: &str) -> egui::Rect {
    let title = h.get_by_label("Lưu thay đổi?").rect();
    let found: Vec<_> = h.query_all_by_role_and_label(Role::Button, label).filter(|b| b.rect().min.y > title.max.y).collect();
    assert_eq!(found.len(), 1, "one {label} in the prompt");
    found[0].rect()
}

fn focused(h: &H, label: &str) -> bool {
    let title = h.get_by_label("Lưu thay đổi?").rect();
    h.query_all_by_role_and_label(Role::Button, label).any(|b| b.rect().min.y > title.max.y && b.is_focused())
}

#[test]
fn embedded_prompt_reads_like_every_nexus_app() {
    let (h, path) = asking("look", true);
    h.get_by_label("Lưu thay đổi?");
    h.get_by_label(&format!("Lưu thay đổi của “{NAME}” trước khi đóng?"));
    let (save, dont, cancel) = (button(&h, "Lưu"), button(&h, "Không lưu"), button(&h, "Hủy"));
    assert!(save.max.x <= dont.min.x && dont.max.x <= cancel.min.x, "Lưu, Không lưu, Hủy: {save:?} {dont:?} {cancel:?}");
    assert!((save.center().y - cancel.center().y).abs() < 0.5, "one row");
    for b in [save, dont, cancel] {
        assert!(b.width() >= BUTTON_SIZE.x - 0.5 && b.height() >= BUTTON_SIZE.y - 0.5, "{b:?}");
    }
    let title = h.get_by_label("Lưu thay đổi?").rect();
    let inner = WIDTH - 2.0 * f32::from(PADDING);
    assert!((cancel.max.x - (title.min.x + inner)).abs() < 1.5, "right-aligned: {cancel:?} {title:?}");
    assert!(focused(&h, "Lưu"), "the keyboard starts on Lưu");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn embedded_prompt_enter_saves_and_escape_cancels() {
    for (key, saves) in [(egui::Key::Enter, true), (egui::Key::Escape, false)] {
        let (mut h, path) = asking(if saves { "enter" } else { "escape" }, true);
        let before = std::fs::read(&path).unwrap();
        h.key_press(key);
        h.run_steps(4);
        let a = h.state();
        assert!(a.dialog.is_none(), "{key:?}: answered");
        assert_eq!(std::fs::read(&path).unwrap() != before, saves, "{key:?}: file written");
        let open = a.session.documents().iter().any(|d| d.display_title() == NAME);
        assert_eq!(open, !saves, "{key:?}: the workbook closes only after Lưu");
        if !saves {
            assert!(a.session.active().is_some_and(|d| d.is_dirty()), "Hủy keeps the changes");
        }
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

#[test]
fn embedded_prompt_dont_save_closes_without_writing() {
    let (mut h, path) = asking("dont", true);
    let before = std::fs::read(&path).unwrap();
    let title = h.get_by_label("Lưu thay đổi?").rect();
    h.query_all_by_role_and_label(Role::Button, "Không lưu").find(|b| b.rect().min.y > title.max.y).unwrap().click();
    h.run_steps(4);
    assert!(h.state().dialog.is_none());
    assert!(!h.state().session.documents().iter().any(|d| d.display_title() == NAME), "closed");
    assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// Stand-alone keeps upstream's prompt.
#[test]
fn stand_alone_prompt_is_upstreams() {
    let (h, path) = asking("standalone", false);
    assert!(h.query_by_label_contains("trước khi đóng").is_none(), "upstream's question, not the NexusOffice one");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
