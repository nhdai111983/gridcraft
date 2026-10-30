use super::{Dialog, Json, SheetApp, json};
use crate::nexus_i18n::{t, tf};

pub(super) fn spelling(app: &mut SheetApp, ui: &mut egui::Ui, d: &mut Dialog, open: &mut bool) {
    let r = d.result.clone().unwrap_or(Json::Null);
    if let Some(e) = r.get("error").and_then(Json::as_str) {
        ui.label(app.ui.language.tr_error(e));
        return;
    }
    let issues = r["issues"].as_array().cloned().unwrap_or_default();
    let Some(it) = issues.get(d.list_index) else {
        ui.label(t(ui, "The spelling check is complete for the entire sheet."));
        if ui.button(t(ui, "OK")).clicked() {
            *open = false;
        }
        return;
    };
    let word = it["word"].as_str().unwrap_or("").to_string();
    let cell = it["cell"].as_str().unwrap_or("").to_string();
    ui.label(tf(ui, "Not in Dictionary ({}):", &[&cell]));
    ui.label(egui::RichText::new(&word).strong().color(crate::theme::Tokens::get(ui.ctx()).danger));
    let sugg: Vec<String> =
        it["suggestions"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
    ui.label(t(ui, "Suggestions:"));
    let mut pick = d.values.get("pick").and_then(Json::as_str).unwrap_or("").to_string();
    for s in &sugg {
        if ui.selectable_label(pick == *s, s).clicked() {
            pick = s.clone();
        }
    }
    if sugg.is_empty() {
        ui.label(egui::RichText::new(t(ui, "(No suggestions)")).italics());
    }
    d.values.insert("pick".into(), json!(pick.clone()));
    let _ = app.session.run("selection.set", json!({"cell": cell}));
    ui.horizontal(|ui| {
        if ui.button(t(ui, "Ignore Once")).clicked() {
            d.list_index += 1;
        }
        if ui.button(t(ui, "Add to Dictionary")).clicked() {
            let _ = app.session.run("review.addToDictionary", json!({"word": word}));
            d.list_index += 1;
        }
        if !pick.is_empty() {
            if ui.button(t(ui, "Change")).clicked() {
                app.run_or_alert("review.changeSpelling", json!({"cell": cell, "word": word, "to": pick}));
                d.list_index += 1;
            }
            if ui.button(t(ui, "Change All")).clicked() {
                app.run_or_alert("review.changeSpelling", json!({"word": word, "to": pick, "all": true}));
                d.list_index += 1;
            }
        }
    });
}
