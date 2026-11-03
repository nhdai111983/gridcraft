//! NexusOffice: what a host window with several editors (the NexusOffice tabs) needs from the app —
//! drafts settled before a tab closes, fonts that live beside another editor's — and the Nexus
//! preferences applied each frame (sheet names in the interface's language, the accent colour).

use super::{SheetApp, clean_error, editor, i18n, text_box, theme};
use crate::nexus;

impl SheetApp {
    /// NexusOffice: text being typed that would change the workbook when entered — a cell edit
    /// that changed the cell's text, or a text box whose text changed. An edit opened with F2 (or a
    /// double-click) and left as it was is not a change.
    pub fn has_unsaved_edit(&self) -> bool {
        self.editor.as_ref().is_some_and(editor::EditState::changed) || self.text_box_editor.as_ref().is_some_and(text_box::EditState::changed)
    }

    /// NexusOffice: put a draft being typed into the workbook before the host closes it: a changed
    /// cell edit is entered, an unchanged one just ends (no change is recorded), a text box is
    /// committed. `false` when the draft can't be entered (the app shows why).
    pub fn commit_drafts(&mut self) -> bool {
        match self.editor.as_ref().map(editor::EditState::changed) {
            Some(false) => self.cancel_edit(),
            Some(true) if !self.commit_edit(0, 0, false, false) => return false,
            _ => {}
        }
        if let Err(e) = self.commit_text_box_edit() {
            self.message = Some((nexus::APP_NAME.into(), clean_error(&e)));
            return false;
        }
        true
    }

    /// NexusOffice: install the fonts and the look again on the next frame (painting nothing on
    /// this one). A host window with other editors calls it when this app's tab comes forward,
    /// since another editor may have replaced the window's fonts and visuals meanwhile.
    pub fn restyle(&mut self) {
        self.fonts_ready = false;
        self.fonts_set = false;
    }

    /// Whether the window's fonts have this app's interface family (another editor in a host
    /// window may have replaced them).
    pub(super) fn fonts_installed(&self, ctx: &egui::Context) -> bool {
        let ui = egui::FontFamily::Name(theme::UI.into());
        ctx.fonts(|f| f.definitions().families.contains_key(&ui))
    }

    /// Installs `defs`. Embedded, the window's other families (another editor's, the host's own
    /// proportional and monospace faces) are kept and this app's named families added, so the
    /// editors in a host window's tabs don't take each other's fonts away.
    pub(super) fn set_fonts(&self, ctx: &egui::Context, defs: egui::FontDefinitions) {
        let defs = with_ui_fallback(defs);
        if !self.embedded {
            return ctx.set_fonts(defs);
        }
        let mut merged = ctx.fonts(|f| f.definitions().clone());
        merged.font_data.extend(defs.font_data);
        for (family, chain) in defs.families {
            if matches!(family, egui::FontFamily::Name(_)) {
                merged.families.insert(family, chain);
            }
        }
        // The host's Proportional faces stay first; ours follow for the letters they lack.
        ctx.set_fonts(with_ui_fallback(merged));
    }

    /// The grid and the cell editor own Tab, the arrows and Escape, but egui learns that only from
    /// a widget that held the focus in the previous frame (`set_focus_lock_filter`). In the frame
    /// right after one of them took the focus (a character typed starts the editor; committing
    /// gives the keyboard back to the grid) egui reads those keys as "move the focus": Tab right
    /// after a character, or right after Enter/Tab committed an entry, sent egui's focus to
    /// another widget and the keys typed next never reached the grid. Cancel that move before
    /// any widget is drawn: the key is ours.
    pub(super) fn keep_keyboard(&self, ctx: &egui::Context) {
        let ours = [Some(egui::Id::unique("gridcraft.cell_editor")), Some(egui::Id::unique("gridcraft.formula_bar")), self.grid.focus_id];
        if self.dialog.is_none() && self.message.is_none() && ctx.memory(|m| m.focused()).is_some_and(|f| ours.contains(&Some(f))) {
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }
    }

    /// New sheets are named in the interface's language (Trang tính1), and the user's accent colour
    /// (File › Options, `ui.accent`) is applied.
    pub(super) fn sync_nexus_preferences(&mut self, ctx: &egui::Context) {
        let prefix = if self.ui.language == i18n::Language::Vi { "Trang tính" } else { "" };
        if self.session.sheet_name_prefix != prefix {
            self.session.sheet_name_prefix = prefix.to_string();
        }
        // Workbooks show in the interface language's regional format (vi-VN: 1.234,5, dd/mm/yyyy);
        // cheap when nothing changed, and an undone step that comes back with another locale gets
        // this one again.
        self.session.set_locale(self.ui.regional_format());
        theme::set_accent(ctx, self.ui.accent.as_deref().and_then(theme::parse_hex));
    }
}

/// NexusOffice: egui's own Proportional family falls back on the interface's faces, so text drawn
/// in it (the shared "Lưu thay đổi?" prompt, `FontId::proportional`) has the Vietnamese letters the
/// bundled face lacks (`ủ`, `ộ`…) instead of boxes.
fn with_ui_fallback(mut defs: egui::FontDefinitions) -> egui::FontDefinitions {
    let Some(ui) = defs.families.get(&egui::FontFamily::Name(theme::UI.into())).cloned() else { return defs };
    let chain = defs.families.entry(egui::FontFamily::Proportional).or_default();
    for face in ui {
        if !chain.contains(&face) {
            chain.push(face);
        }
    }
    defs
}
