//! NexusOffice: the regional format (File › Options › Region), set apart from the interface
//! language as Windows and Excel keep them apart: Vietnamese menus with en-US numbers, or English
//! menus with `1.234,5`, `25/10/2026` and đồng.
//!
//! Unset, the region follows the interface language (Vietnamese → vi-VN, every other language →
//! en-US). The choice is saved with the preferences (`ui.json`, key `region`). The formula syntax
//! goes with the region (vi-VN: decimal comma, `;` between arguments), except Spanish, which keeps
//! its own function names and separators. Programmatic calls (control channel, MCP, CLI) stay
//! en-US unless they name a `locale` (unchanged).
//!
//! Also here: format codes in the Format Cells dialog, shown and edited in the regional format
//! and stored en-US ([`CodeEdit`]).

use gridcraft_engine::core::Locale;
use gridcraft_engine::formula::FormulaLocale;
use serde::Deserialize;
use serde_json::{Value as Json, json};

use crate::UiState;
use crate::i18n::Language;

impl UiState {
    /// The regional format the workbooks are shown, typed and coerced in: the chosen region, or
    /// the interface language's.
    pub fn regional_format(&self) -> Locale {
        regional_format(self.language, self.region)
    }

    /// The formula syntax typed and shown in the formula bar and grid.
    pub fn formula_locale(&self) -> FormulaLocale {
        formula_locale(self.language, self.region)
    }
}

/// [`UiState::regional_format`] for a language and a saved region.
pub fn regional_format(language: Language, region: Option<Locale>) -> Locale {
    region.unwrap_or_else(|| language.regional_format())
}

/// [`UiState::formula_locale`]: Spanish keeps its names and separators; otherwise the region
/// decides (vi-VN: English names, `;`, decimal comma).
pub fn formula_locale(language: Language, region: Option<Locale>) -> FormulaLocale {
    if language.formula_locale() == FormulaLocale::Es {
        return FormulaLocale::Es;
    }
    match regional_format(language, region) {
        Locale::ViVn => FormulaLocale::Vi,
        Locale::EnUs => FormulaLocale::En,
    }
}

/// Reads a saved region leniently: `"vi-VN"`/`"en-US"` (any case or `_`), anything else (a region
/// removed later, a hand-edited file) is "follow the language" rather than an error that would
/// reset every other preference.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Locale>, D::Error> {
    let v = Json::deserialize(d)?;
    Ok(v.as_str().and_then(Locale::from_tag))
}

/// Writes a saved region as its tag.
pub fn serialize<S: serde::Serializer>(region: &Option<Locale>, s: S) -> Result<S::Ok, S::Error> {
    match region {
        Some(l) => s.serialize_str(l.tag()),
        None => s.serialize_none(),
    }
}

/// The `ui.region` command: `{"value": "vi-VN" | "en-US"}` sets the region, `""` or `null`
/// follows the interface language again, no `value` reads it. Returns
/// `{region: tag | null, effective: tag}`.
pub fn command(ui: &mut UiState, p: &Json) -> Result<Json, String> {
    if let Some(v) = p.get("value") {
        match v.as_str().map(str::trim) {
            None if v.is_null() => ui.region = None,
            Some("") => ui.region = None,
            Some(tag) => match Locale::from_tag(tag) {
                Some(l) => ui.region = Some(l),
                None => {
                    let shown: String = tag.chars().take(24).collect();
                    return Err(format!("unknown region {shown:?}; use \"vi-VN\", \"en-US\", or \"\" to follow the interface language"));
                }
            },
            None => return Err("the region is a string: \"vi-VN\", \"en-US\", or \"\" to follow the interface language".into()),
        }
    }
    Ok(json!({"region": ui.region.map(Locale::tag), "effective": ui.regional_format().tag()}))
}

/// A region's name in its own language, as region pickers show it.
pub fn region_name(l: Locale) -> &'static str {
    match l {
        Locale::ViVn => "Tiếng Việt (Việt Nam)",
        Locale::EnUs => "English (United States)",
    }
}

/// The choices of File › Options › Region: follow the language, then each region.
pub fn choices(language: Language) -> Vec<(Option<Locale>, String)> {
    let follow = crate::i18n::nexus_text(language, "Same as interface language", "Theo ngôn ngữ giao diện");
    let mut v = vec![(None, format!("{follow} — {}", region_name(language.regional_format())))];
    v.extend(Locale::ALL.iter().map(|l| (Some(*l), region_name(*l).to_string())));
    v
}

/// The label of File › Options › Region.
pub fn label(language: Language) -> &'static str {
    crate::i18n::nexus_text(language, "Region (numbers and dates)", "Vùng (số và ngày)")
}

/// File › Options › General: the Region row (a label and a combo box) in the options grid.
pub fn options_row(app: &mut crate::SheetApp, ui: &mut egui::Ui) {
    let lang = app.ui.language;
    ui.label(label(lang));
    let current = app.ui.region;
    let choices = choices(lang);
    let shown = choices.iter().find(|(r, _)| *r == current).map(|(_, n)| n.clone()).unwrap_or_default();
    egui::ComboBox::from_id_salt("backstage_region").width(260.0).selected_text(shown).show_ui(ui, |ui| {
        for (region, name) in &choices {
            if ui.selectable_label(*region == current, name.as_str()).clicked() {
                app.run_or_alert("ui.region", json!({"value": region.map(Locale::tag)}));
            }
        }
    });
    ui.end_row();
}

// ------------------------------------------------------------------------------- per frame

fn formula_locale_key() -> egui::Id {
    egui::Id::new("nexus_formula_locale")
}

/// Sets the formula syntax for the current frame, for painters that only have a `Context`.
pub fn set_current(ctx: &egui::Context, locale: FormulaLocale) {
    ctx.data_mut(|d| d.insert_temp(formula_locale_key(), locale));
}

/// The formula syntax [`set_current`] stored for this frame (English when unset).
pub fn current(ctx: &egui::Context) -> FormulaLocale {
    ctx.data(|d| d.get_temp(formula_locale_key())).unwrap_or_default()
}

// ------------------------------------------------------------------- Format Cells codes

/// A format code being edited in the Format Cells dialog: the stored (en-US) code and the text
/// shown for it in the regional format. Excel's dialog shows codes the way the system writes
/// them (`#.##0,00`, `#.##0 [$₫-42A]`, the system short date `dd/mm/yyyy`); the file keeps
/// `#,##0.00`, `#,##0 [$₫-42A]`, `m/d/yyyy`.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeEdit {
    /// The en-US code the cell gets.
    pub stored: String,
    /// What the dialog's text field shows and edits.
    pub shown: String,
}

impl CodeEdit {
    /// Starts editing a stored code.
    pub fn new(stored: &str, locale: Locale) -> CodeEdit {
        CodeEdit { stored: stored.to_string(), shown: show(stored, locale) }
    }

    /// The code to store for `shown` as now edited: the original when the text is still what it
    /// was shown as (so opening the dialog never rewrites a code), and otherwise the text read the
    /// regional format's way ([`store`]).
    pub fn edited(&mut self, locale: Locale) {
        if self.shown != show(&self.stored, locale) {
            self.stored = store(&self.shown, locale);
        }
    }

    /// Picks a stored code (a category or a preset button).
    pub fn pick(&mut self, stored: &str, locale: Locale) {
        *self = CodeEdit::new(stored, locale);
    }
}

/// A stored code as the dialog shows it.
pub fn show(stored: &str, locale: Locale) -> String {
    gridcraft_engine::numfmt::localize_code(stored, locale)
}

/// Text typed in the dialog as the code to store. The locale's spelling of the system short date
/// (`dd/mm/yyyy` in vi-VN, with or without ` h:mm`) is the system date itself (built-in 14/22),
/// as Excel's list shows the system short date in the system's order.
pub fn store(shown: &str, locale: Locale) -> String {
    let t = shown.trim();
    for system in ["m/d/yyyy", "m/d/yyyy h:mm"] {
        if false && locale != Locale::EnUs && t == show(system, locale) {
            return system.to_string();
        }
    }
    gridcraft_engine::numfmt::delocalize_code(shown, locale)
}

/// The Number Format categories, in the order the dialog and the ribbon list them.
pub const CATEGORIES: [&str; 11] =
    ["General", "Number", "Currency", "Accounting", "Short Date", "Long Date", "Time", "Percentage", "Fraction", "Scientific", "Text"];

/// The code a Number Format category applies in a region: in vi-VN Currency is đồng's
/// (`1.234.567 ₫`, no decimals), as Excel's is the system currency's. Unknown names are General.
pub fn category_code(region: Locale, name: &str) -> &'static str {
    if name == "Currency" && region == Locale::ViVn {
        return gridcraft_engine::core::parse::VND_FORMAT;
    }
    CATEGORIES.iter().find(|n| **n == name).map(|n| gridcraft_engine::cmd::format::format_code_for(n)).unwrap_or("General")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_follows_the_language_until_chosen() {
        assert_eq!(regional_format(Language::Vi, None), Locale::ViVn);
        assert_eq!(regional_format(Language::En, None), Locale::EnUs);
        assert_eq!(regional_format(Language::En, Some(Locale::ViVn)), Locale::ViVn);
        assert_eq!(regional_format(Language::Vi, Some(Locale::EnUs)), Locale::EnUs);
        assert_eq!(formula_locale(Language::En, Some(Locale::ViVn)), FormulaLocale::Vi);
        assert_eq!(formula_locale(Language::Vi, Some(Locale::EnUs)), FormulaLocale::En);
        assert_eq!(formula_locale(Language::Es, Some(Locale::ViVn)), FormulaLocale::Es);
        assert_eq!(formula_locale(Language::Es, None), FormulaLocale::Es);
        assert_eq!(formula_locale(Language::Ja, None), FormulaLocale::En);
    }

    #[test]
    fn the_region_command_sets_reads_and_clears() {
        let mut ui = UiState { language: Language::En, ..UiState::default() };
        assert_eq!(command(&mut ui, &json!({})).unwrap(), json!({"region": null, "effective": "en-US"}));
        assert_eq!(command(&mut ui, &json!({"value": "vi-VN"})).unwrap(), json!({"region": "vi-VN", "effective": "vi-VN"}));
        assert_eq!(command(&mut ui, &json!({"value": " vi_vn "})).unwrap()["region"], json!("vi-VN"));
        assert_eq!(command(&mut ui, &json!({"value": ""})).unwrap()["region"], json!(null));
        command(&mut ui, &json!({"value": "en-US"})).unwrap();
        assert_eq!(command(&mut ui, &json!({"value": null})).unwrap()["region"], json!(null));
        // Junk is an error that leaves the setting alone; nothing panics.
        command(&mut ui, &json!({"value": "vi-VN"})).unwrap();
        for junk in [json!("fr-FR"), json!(5), json!(true), json!([1]), json!({"a": 1}), json!("x".repeat(100_000)), json!("\u{0}")] {
            assert!(command(&mut ui, &json!({"value": junk})).is_err());
            assert_eq!(ui.region, Some(Locale::ViVn));
        }
    }

    #[test]
    fn the_region_is_saved_and_read_leniently() {
        let ui = UiState { region: Some(Locale::ViVn), ..UiState::default() };
        let saved = serde_json::to_value(&ui).unwrap();
        assert_eq!(saved["region"], json!("vi-VN"));
        let back: UiState = serde_json::from_value(saved).unwrap();
        assert_eq!(back.region, Some(Locale::ViVn));
        // Unset is not written; junk reads as unset without losing the other preferences.
        assert!(serde_json::to_value(UiState::default()).unwrap().get("region").is_none());
        for junk in [json!("xx"), json!(7), json!(null), json!({"x": 1})] {
            let back: UiState = serde_json::from_value(json!({"region": junk, "language": "vi", "animations": false})).unwrap();
            assert_eq!(back.region, None);
            assert_eq!(back.language, Language::Vi);
            assert!(!back.animations);
        }
    }

    #[test]
    fn format_codes_show_in_the_region_and_store_en_us() {
        use Locale::{EnUs, ViVn};
        // Opening and closing the dialog keeps every code as it was.
        for c in ["#,##0.00", "m/d/yyyy", "m/d/yyyy h:mm", "#,##0 [$₫-42A]", "General", "[$-F800]dddd, mmmm dd, yyyy", "0.00E+00", "@"] {
            for l in Locale::ALL {
                let mut e = CodeEdit::new(c, l);
                e.edited(l);
                assert_eq!(e.stored, c, "{c} in {l:?}");
            }
        }
        let mut e = CodeEdit::new("#,##0.00", ViVn);
        assert_eq!(e.shown, "#.##0,00");
        e.shown = "#.##0,000".into();
        e.edited(ViVn);
        assert_eq!(e.stored, "#,##0.000");
        let mut e = CodeEdit::new("m/d/yyyy", ViVn);
        assert_eq!(e.shown, "dd/mm/yyyy");
        e.shown = "dd/mm/yy".into();
        e.edited(ViVn);
        assert_eq!(e.stored, "dd/mm/yy");
        // Typed back, the locale's short date is the system date again (built-in 14, 22).
        e.shown = "dd/mm/yyyy".into();
        e.edited(ViVn);
        assert_eq!(e.stored, "m/d/yyyy");
        e.shown = " dd/mm/yyyy h:mm".into();
        e.edited(ViVn);
        assert_eq!(e.stored, "m/d/yyyy h:mm");
        assert_eq!(store("dd/mm/yyyy", EnUs), "dd/mm/yyyy");
        // A code with a comma in its condition survives an edit elsewhere in it.
        let mut e = CodeEdit::new("[>=1,000]#,##0;0.0", ViVn);
        assert_eq!(e.shown, "[>=1.000]#.##0;0,0");
        e.shown = "[>=1.000]#.##0;0,00".into();
        e.edited(ViVn);
        assert_eq!(e.stored, "[>=1,000]#,##0;0.00");
        e.pick(category_code(ViVn, "Currency"), ViVn);
        assert_eq!((e.stored.as_str(), e.shown.as_str()), ("#,##0 [$₫-42A]", "#.##0 [$₫-42A]"));
        // en-US shows and stores the code as written.
        let mut e = CodeEdit::new("#,##0.00", EnUs);
        assert_eq!(e.shown, "#,##0.00");
        e.shown = "#,##0.0".into();
        e.edited(EnUs);
        assert_eq!(e.stored, "#,##0.0");
        assert_eq!(category_code(EnUs, "Currency"), gridcraft_engine::cmd::format::format_code_for("Currency"));
        assert_eq!(category_code(ViVn, "Short Date"), "m/d/yyyy");
        assert_eq!(category_code(ViVn, "nonsense"), "General");
        // Hostile text never panics.
        for junk in ["[", "\"", "\\", "[<1,5", ";;;;", "\u{0}", "ẮẰẲẴẶ"] {
            let mut e = CodeEdit::new("0", ViVn);
            e.shown = junk.to_string();
            e.edited(ViVn);
            let _ = show(&e.stored, ViVn);
        }
    }
}
