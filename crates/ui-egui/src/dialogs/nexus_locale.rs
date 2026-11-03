//! NexusOffice: dialogs take numbers and dates the way the interface types them (vi-VN: `1,5`,
//! `24/10/2026`) and send their commands the `locale` to read them in.

use super::Json;
use serde_json::json;

/// Commands whose typed values a dialog sends in the interface's regional format (`locale`).
pub(super) const LOCALE_COMMANDS: &[&str] = &[
    "data.textToColumns",
    "data.forecastSheet",
    "data.solver",
    "data.scenarioAdd",
    "data.getData",
    "data.fromTextCsv",
    "data.filterBy",
    "cell.set",
    "range.fill",
];

/// The solver dialog's saved numbers (Value Of, constraint right sides) in `locale`'s spelling.
pub(super) fn localize_solver_numbers(values: &mut Json, locale: gridcraft_engine::core::Locale) {
    if locale == gridcraft_engine::core::Locale::EnUs {
        return;
    }
    if let Some(v) = values.get_mut("value")
        && let Some(t) = v.as_str().map(str::trim).filter(|t| t.parse::<f64>().is_ok()).map(str::to_string)
    {
        *v = json!(locale.localize_number(&t));
    }
    if let Some(Json::Array(list)) = values.get_mut("constraints") {
        for c in list.iter_mut() {
            let shown = c
                .as_str()
                .and_then(|t| t.rsplit_once(' '))
                .filter(|(_, tail)| tail.parse::<f64>().is_ok())
                .map(|(head, tail)| format!("{head} {}", locale.localize_number(tail)));
            if let Some(shown) = shown {
                *c = json!(shown);
            }
        }
    }
}
