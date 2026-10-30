//! NexusOffice (QC-SHT-08): the Review and Formulas reports (Workbook Statistics, Check
//! Accessibility, Error Checking, Evaluate Formula, the comment list and the action log) drawn as
//! labelled tables in the interface's language, not as raw JSON or command logs.

use egui::Ui;
use serde_json::Value as Json;

use crate::nexus_i18n::{t, tf};

/// Draws the report of dialog `name` from its command result.
pub fn report(ui: &mut Ui, name: &str, result: &Json) {
    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| match name {
        "statistics" => statistics(ui, result),
        "accessibility" => accessibility(ui, result),
        "errorChecking" => error_checking(ui, result),
        "evaluateFormula" => evaluate_formula(ui, result),
        "comments" => comments(ui, result),
        _ => {
            ui.label(text(result));
        }
    });
}

/// A value as shown in a report cell: text as is, numbers plainly, errors by name.
fn text(v: &Json) -> String {
    match v {
        Json::Null => "—".into(),
        Json::String(s) => s.clone(),
        Json::Number(n) => n.to_string(),
        Json::Bool(b) => b.to_string().to_uppercase(),
        Json::Object(o) => o.get("error").and_then(Json::as_str).map(str::to_string).unwrap_or_default(),
        Json::Array(a) => a.iter().map(text).collect::<Vec<_>>().join(", "),
    }
}

fn rows(ui: &mut Ui, id: &str, rows: &[(&str, String)]) {
    egui::Grid::new(id).num_columns(2).spacing(egui::vec2(16.0, 4.0)).striped(true).show(ui, |ui| {
        for (label, value) in rows {
            ui.label(t(ui, label));
            ui.label(value);
            ui.end_row();
        }
    });
}

fn statistics(ui: &mut Ui, r: &Json) {
    let sheet = &r["sheet"];
    let wb = &r["workbook"];
    ui.strong(tf(ui, "Current sheet: {}", &[&text(&sheet["name"])]));
    rows(
        ui,
        "stats_sheet",
        &[
            ("End of sheet", text(&sheet["endOfSheet"])),
            ("Cells with data", text(&sheet["cellsWithData"])),
            ("Tables", text(&sheet["tables"])),
            ("Formulas", text(&sheet["formulas"])),
            ("Charts", text(&sheet["charts"])),
            ("Comments", text(&sheet["comments"])),
        ],
    );
    ui.add_space(6.0);
    ui.strong(t(ui, "Workbook"));
    rows(
        ui,
        "stats_workbook",
        &[
            ("Sheets", text(&wb["sheets"])),
            ("Cells with data", text(&wb["cellsWithData"])),
            ("Tables", text(&wb["tables"])),
            ("Charts", text(&wb["charts"])),
            ("Names", text(&wb["names"])),
        ],
    );
}

fn accessibility(ui: &mut Ui, r: &Json) {
    let issues = r.as_array().map(Vec::as_slice).unwrap_or_default();
    if issues.is_empty() {
        ui.label(t(ui, "No accessibility issues found."));
        return;
    }
    egui::Grid::new("a11y").num_columns(3).spacing(egui::vec2(16.0, 4.0)).striped(true).show(ui, |ui| {
        ui.strong(t(ui, "Sheet"));
        ui.strong(t(ui, "Issue"));
        ui.strong(t(ui, "Where"));
        ui.end_row();
        for i in issues {
            ui.label(text(&i["sheet"]));
            ui.label(t(ui, i["issue"].as_str().unwrap_or_default()));
            let at = match (i.get("object").and_then(Json::as_str), i.get("count")) {
                (Some(o), _) => o.replacen("Picture", t(ui, "Picture"), 1),
                (None, Some(n)) => tf(ui, "{} ranges", &[&text(n)]),
                _ => String::new(),
            };
            ui.label(at);
            ui.end_row();
        }
    });
}

fn error_checking(ui: &mut Ui, r: &Json) {
    let errors = r["errors"].as_array().map(Vec::as_slice).unwrap_or_default();
    let circular = r["circular"].as_array().map(Vec::as_slice).unwrap_or_default();
    if errors.is_empty() && circular.is_empty() {
        ui.label(t(ui, "No errors found on this sheet."));
        return;
    }
    if !errors.is_empty() {
        egui::Grid::new("errors").num_columns(3).spacing(egui::vec2(16.0, 4.0)).striped(true).show(ui, |ui| {
            ui.strong(t(ui, "Cell"));
            ui.strong(t(ui, "Error"));
            ui.strong(t(ui, "Formula"));
            ui.end_row();
            for e in errors {
                ui.label(format!("{}!{}", text(&e["sheet"]), text(&e["cell"])));
                ui.label(text(&e["error"]));
                ui.label(text(&e["formula"]));
                ui.end_row();
            }
        });
    }
    if !circular.is_empty() {
        ui.add_space(6.0);
        ui.label(tf(ui, "Circular References: {}", &[&circular.iter().map(text).collect::<Vec<_>>().join(", ")]));
    }
}

fn evaluate_formula(ui: &mut Ui, r: &Json) {
    if r["formula"].is_null() {
        ui.label(t(ui, "Select a cell with a formula."));
        return;
    }
    rows(ui, "eval_head", &[("Formula", text(&r["formula"])), ("Result", text(&r["result"]))]);
    let steps = r["steps"].as_array().map(Vec::as_slice).unwrap_or_default();
    if !steps.is_empty() {
        ui.add_space(6.0);
        ui.strong(t(ui, "Steps"));
        egui::Grid::new("eval_steps").num_columns(2).spacing(egui::vec2(16.0, 4.0)).striped(true).show(ui, |ui| {
            for s in steps {
                ui.monospace(text(&s["expression"]));
                ui.label(text(&s["value"]));
                ui.end_row();
            }
        });
    }
}

fn comments(ui: &mut Ui, r: &Json) {
    let list = r.as_array().map(Vec::as_slice).unwrap_or_default();
    if list.is_empty() {
        ui.label(t(ui, "No comments on this sheet yet. Select a cell and choose New Comment."));
        return;
    }
    egui::Grid::new("comments_list").num_columns(3).spacing(egui::vec2(16.0, 4.0)).striped(true).show(ui, |ui| {
        ui.strong(t(ui, "Cell"));
        ui.strong(t(ui, "Author"));
        ui.strong(t(ui, "Comment"));
        ui.end_row();
        for c in list {
            ui.label(text(&c["cell"]));
            ui.label(text(&c["author"]));
            ui.label(text(&c["text"]));
            ui.end_row();
        }
    });
}

/// The action log (Automate › Action Log): one line per command, its menu label translated.
pub fn journal(ui: &mut Ui, entries: &[(String, Json)]) {
    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
        if entries.is_empty() {
            ui.label(t(ui, "No actions recorded yet."));
        }
        egui::Grid::new("journal").num_columns(2).spacing(egui::vec2(16.0, 2.0)).striped(true).show(ui, |ui| {
            for (id, p) in entries.iter().rev().take(500) {
                let label = gridcraft_engine::find_command(id).map(|c| c.label).unwrap_or(id.as_str());
                ui.label(t(ui, label)).on_hover_text(id.as_str());
                let shown = match p {
                    Json::Null => String::new(),
                    Json::Object(o) if o.is_empty() => String::new(),
                    p => p.to_string(),
                };
                ui.monospace(shown);
                ui.end_row();
            }
        });
    });
}
