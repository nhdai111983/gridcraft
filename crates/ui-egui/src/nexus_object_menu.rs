//! NexusOffice (QC-SHT-11): right-clicking a chart, picture or shape opens that object's menu
//! (Change Chart Type, Select Data, Format, order, Delete), not the cell menu underneath it. The
//! menu itself is upstream's (`grid::menus::object_menu`, #2067); this module supplies its items.

use serde_json::json;

use crate::SheetApp;

/// The items of an object's menu: `(label, action)`; `"-"` is a separator.
pub fn items(kind: &str) -> Vec<(&'static str, &'static str)> {
    let mut v = Vec::new();
    if kind == "chart" {
        v.extend([("Change Chart Type…", "type"), ("Select Data…", "data"), ("Format Chart Area…", "format"), ("-", "")]);
    }
    v.extend([("Bring Forward", "forward"), ("Send Backward", "backward"), ("-", ""), ("Delete", "delete")]);
    v
}

/// Runs a menu item for object `kind`/`id`.
pub fn run(app: &mut SheetApp, kind: &str, id: u32, action: &str) {
    app.selected_chart = Some(id);
    match action {
        // The Format Chart pane holds the chart types.
        "type" | "format" => app.grid.pane = Some("formatChart".into()),
        "data" => app.open_dialog("selectData", json!({"chart": id})),
        "forward" => app.run_or_alert("arrange.bringForward", json!({"kind": kind, "id": id})),
        "backward" => app.run_or_alert("arrange.sendBackward", json!({"kind": kind, "id": id})),
        "delete" => {
            app.run_or_alert("object.delete", json!({"kind": kind, "id": id}));
            app.selected_chart = None;
        }
        _ => {}
    }
}
