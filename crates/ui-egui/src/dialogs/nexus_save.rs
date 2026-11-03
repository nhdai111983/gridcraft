//! NexusOffice: "Lưu thay đổi?" over the tab (`nexus_save_prompt`), the prompt every Nexus app
//! shows as a tab of the NexusOffice window. Lưu, Không lưu and Hủy answer as the stand-alone
//! prompt's Save, Don't Save and Cancel do ([`SheetApp::answer_save_changes`]).

use super::{Dialog, Json, SheetApp};
use crate::nexus_save_prompt as sp;

/// Shows the embedded prompt for `d` (a `saveChanges` dialog); puts the dialog back until it is
/// answered.
pub(super) fn embedded_save_prompt(app: &mut SheetApp, ctx: &egui::Context, d: Dialog) {
    let t = crate::theme::Tokens::get(ctx);
    let vi = app.ui.language == crate::i18n::Language::Vi;
    let name = d.values.get("title").and_then(Json::as_str).unwrap_or_default();
    let question = sp::question(vi, name);
    let prompt = sp::Prompt { words: sp::words(vi), question: &question, note: None, error: None };
    let look = sp::Look {
        dark: t.dark,
        accent: t.accent,
        on_accent: t.on_accent,
        error: ctx.global_style().visuals.error_fg_color,
        title_font: egui::FontId::new(sp::TITLE_SIZE, egui::FontFamily::Name(crate::theme::UI_BOLD.into())),
        title_fake_bold: false,
    };
    let quitting = d.values.get("quit").and_then(Json::as_bool) == Some(true);
    let answer = match sp::show(ctx, egui::Id::unique("nexus_save_prompt"), app.drawn_in, &prompt, &look) {
        None => {
            app.dialog = Some(d);
            return;
        }
        Some(sp::Answer::Cancel) => crate::SaveAnswer::Cancel,
        Some(sp::Answer::DontSave) => crate::SaveAnswer::DontSave,
        Some(sp::Answer::Save) => crate::SaveAnswer::Save,
    };
    // Save keeps the prompt when the cell being edited can't be entered.
    if !app.answer_save_changes(answer, quitting, ctx) {
        app.dialog = Some(d);
    }
}
