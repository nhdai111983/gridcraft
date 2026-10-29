//! NexusOffice (QC-SHT-11): right-clicking a chart, picture or shape opens that object's menu
//! (Change Chart Type, Select Data, Format, order, Delete), not the cell menu underneath it.

use egui::{Pos2, vec2};
use serde_json::json;

use crate::SheetApp;
use crate::nexus_i18n::t;

/// The open object menu: where, and which object (`kind` as `chartview::hit` names it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectMenu {
    pub at: Pos2,
    /// The pass it opened in: the right-click that opened it is not a click elsewhere.
    pub pass: u64,
    pub kind: &'static str,
    pub id: u32,
}

fn key() -> egui::Id {
    egui::Id::new("nexus_object_menu")
}

/// Opens the menu for object `kind`/`id` at `at` (screen position).
pub fn open(ctx: &egui::Context, mut menu: ObjectMenu) {
    menu.pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(key(), Some(menu)));
}

/// The open object menu, if any.
pub fn current(ctx: &egui::Context) -> Option<ObjectMenu> {
    ctx.data(|d| d.get_temp::<Option<ObjectMenu>>(key())).flatten()
}

fn close(ctx: &egui::Context) {
    ctx.data_mut(|d| d.insert_temp::<Option<ObjectMenu>>(key(), None));
}

/// The items of an object's menu: `(label, action)`; `"-"` is a separator.
pub fn items(kind: &str) -> Vec<(&'static str, &'static str)> {
    let mut v = Vec::new();
    if kind == "chart" {
        v.extend([("Change Chart Type…", "type"), ("Select Data…", "data"), ("Format Chart Area…", "format"), ("-", "")]);
    }
    v.extend([("Bring Forward", "forward"), ("Send Backward", "backward"), ("-", ""), ("Delete", "delete")]);
    v
}

/// Draws the open object menu; a click elsewhere or Escape closes it.
pub fn show(app: &mut SheetApp, ctx: &egui::Context) {
    let Some(m) = current(ctx) else { return };
    let mut chosen = None;
    let resp = egui::Area::new(egui::Id::new("nexus_object_menu_area")).fixed_pos(m.at).order(egui::Order::Foreground).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.set_min_width(200.0);
            for (label, action) in items(m.kind) {
                if label == "-" {
                    ui.separator();
                    continue;
                }
                if ui.add(egui::Button::new(t(ui, label)).frame(false).min_size(vec2(190.0, 20.0))).clicked() {
                    chosen = Some(action);
                }
            }
        });
    });
    let clicked_outside = resp.response.clicked_elsewhere() && ctx.cumulative_pass_nr() != m.pass;
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if let Some(action) = chosen {
        close(ctx);
        run(app, m, action);
    } else if clicked_outside || escape {
        close(ctx);
    }
}

fn run(app: &mut SheetApp, m: ObjectMenu, action: &str) {
    app.selected_chart = Some(m.id);
    match action {
        // The Format Chart pane holds the chart types.
        "type" | "format" => app.grid.pane = Some("formatChart".into()),
        "data" => app.open_dialog("selectData", json!({"chart": m.id})),
        "forward" => app.run_or_alert("arrange.bringForward", json!({"kind": m.kind, "id": m.id})),
        "backward" => app.run_or_alert("arrange.sendBackward", json!({"kind": m.kind, "id": m.id})),
        "delete" => {
            app.run_or_alert("object.delete", json!({"kind": m.kind, "id": m.id}));
            app.selected_chart = None;
        }
        _ => {}
    }
}
