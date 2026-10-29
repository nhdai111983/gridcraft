//! NexusOffice layout in the style of WPS Office (its "Universal UI"; Henry's approved mockup,
//! 2026-10-24), the same layout as Nexus Docs' so the suite's tabs feel like one product. Only the
//! layout is borrowed: every icon is drawn in code (`icons.rs`) and the colours are the app's own
//! (`theme.rs`).
//!
//! There is no title bar: the NexusOffice window's tab strip (or the system's title bar in the
//! stand-alone app) names the workbook. The app draws two rows:
//!
//! - [`bar`] (36 pt): File (opens Backstage), the Quick Access Toolbar, the ribbon tabs left
//!   aligned (» for the ones that don't fit); on the right the save state (with AutoSave in its
//!   menu), Search, Comments, Share and a "⋯" menu (theme, Options, About).
//! - [`strip`] (56 pt): the current tab's groups as one flat row with no group titles, two rows of
//!   small buttons high (a large button is a small icon over one label line). The groups the
//!   window has no room for fold into a "…" panel at the end of the row instead of being cut off.
//!
//! The groups themselves are upstream's ([`crate::ribbon::tab_groups`]); [`group`], called by the
//! ribbon's own group helper, decides which of them each pass draws.

use egui::{Align2, Color32, Id, Rect, Response, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};
use crate::{SheetApp, widgets};

/// Height of the top row (File, Quick Access, tabs, Share).
pub const BAR_H: f32 = 36.0;
/// Height of the command strip under it: two rows of small buttons, or a large button's 20 px
/// icon over its label ([`widgets::COMPACT_H`] plus 3 pt above and below).
pub const STRIP_H: f32 = widgets::COMPACT_H + 6.0;
/// Height of the top row's buttons and search box.
const CONTROL_H: f32 = 28.0;
/// Width kept at the end of the strip for the "…" button.
const OVERFLOW_W: f32 = 30.0;
/// Width of the » button that holds the tabs the top row has no room for.
const MORE_TABS_W: f32 = 26.0;
/// Below this window width the top row drops the word "Comments" (the icon stays) and narrows
/// the search box, so more tabs fit.
const NARROW_W: f32 = 1200.0;

/// The File (Tệp) button.
pub fn file_button_id() -> Id {
    Id::new("nexus_wps_file")
}
/// The strip's "…" button, shown when groups don't fit.
pub fn overflow_button_id() -> Id {
    Id::new("nexus_wps_overflow")
}
/// The save state ("Saved" / "Not saved"), which opens AutoSave, Save and Save As.
pub fn save_state_id() -> Id {
    Id::new("nexus_wps_save_state")
}
/// The top row's "⋯" menu.
pub fn more_button_id() -> Id {
    Id::new("nexus_wps_more")
}
/// The search box.
pub fn search_id() -> Id {
    Id::new("nexus_wps_search")
}
/// The » button with the tabs that don't fit.
pub fn more_tabs_id() -> Id {
    Id::new("nexus_wps_more_tabs")
}

/// Draw the layout's top panels: the top row, and the command strip unless the ribbon is
/// collapsed (double-click a tab).
pub fn show(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // A contextual tab that went away (the selection left the table or chart) leaves for Home.
    let tabs = crate::ribbon::shown_tabs(app);
    if !tabs.contains(&app.ui.ribbon_tab.as_str()) {
        app.ui.ribbon_tab = "Home".into();
    }
    bar(app, ui, &t, &tabs);
    if !app.ui.ribbon_collapsed {
        strip(app, ui, &t);
    }
}

/// The File button's fill: the accent, faint, over the row (a little stronger under the pointer).
pub(crate) fn file_fill(t: &Tokens, hovered: bool) -> Color32 {
    t.ribbon.lerp_to_gamma(t.accent, if hovered { 0.16 } else { 0.08 })
}

/// File's label on `fill`: the accent, darkened toward the body text as far as 4.5:1 needs.
pub(crate) fn file_ink(t: &Tokens, fill: Color32) -> Color32 {
    [0.0, 0.2, 0.4, 0.6, 0.8]
        .iter()
        .map(|k| t.accent.lerp_to_gamma(t.text, *k))
        .find(|ink| theme::contrast_ratio(*ink, fill) >= 4.5)
        .unwrap_or(t.text)
}

/// The colour "Saved" is written in: green that keeps 4.5:1 on the row in either theme.
fn saved_color(dark: bool) -> Color32 {
    if dark { Color32::from_rgb(0x6E, 0xE7, 0xB7) } else { Color32::from_rgb(0x04, 0x78, 0x57) }
}

fn regular(size: f32) -> egui::FontId {
    theme::ui_font(size)
}

fn strong(size: f32) -> egui::FontId {
    theme::ui_bold(size)
}

/// The top row.
fn bar(app: &mut SheetApp, ui: &mut Ui, t: &Tokens, tabs: &[&'static str]) {
    // Stand-alone on macOS the window draws into the title bar: the row sits beside the traffic
    // lights and moves the window. Embedded, the NexusOffice window's own tab strip does that.
    let mac = cfg!(target_os = "macos") && !app.embedded;
    let height = if mac { BAR_H.max(crate::ribbon::MAC_TITLE_BAR) } else { BAR_H };
    let left = if mac { crate::ribbon::MAC_CONTENT_LEFT.clamp(0.0, f32::from(i8::MAX)) as i8 } else { 12 };
    egui::Panel::top("wps_bar")
        .exact_size(height)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let full = ui.max_rect();
            if mac {
                let drag = ui.interact(full, ui.id().with("wps_drag"), Sense::click_and_drag());
                if drag.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }
            }
            let narrow = ui.ctx().content_rect().width() < NARROW_W;
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                file_button(ui, app, t);
                ui.add_space(8.0);
                crate::ribbon::quick_access(app, ui, CONTROL_H, false);
                ui.add_space(6.0);
                let (d, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
                ui.painter().rect_filled(d, 0.0, t.ribbon_border);
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                    more_menu(ui, app, t);
                    share_button(ui, app, t);
                    comments_button(ui, app, t, narrow);
                    search_box(ui, app, t, narrow);
                    ui.add_space(6.0);
                    save_state(ui, app, t);
                    ui.add_space(12.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                        tab_row(ui, app, t, tabs, full.height());
                    });
                });
            });
            ui.painter().hline(full.x_range(), full.max.y - 0.5, Stroke::new(1.0, t.ribbon_border.gamma_multiply(0.6)));
        });
}

/// File (Tệp): the three bars and the word on a faint accent, opening Backstage as File does.
fn file_button(ui: &mut Ui, app: &mut SheetApp, t: &Tokens) -> Response {
    let label = app.ui.language.tr("File");
    let font = strong(13.0);
    let tw = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.accent).size().x;
    let (r, _) = ui.allocate_exact_size(vec2(10.0 + 16.0 + 6.0 + tw + 12.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, file_button_id(), Sense::click());
    let fill = file_fill(t, resp.hovered());
    let ink = file_ink(t, fill);
    ui.painter().rect_filled(r, 6.0, fill);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(16.0, 16.0)), Icon::Hamburger, ink);
    ui.painter().text(pos2(r.min.x + 32.0, r.center().y), Align2::LEFT_CENTER, label, font, ink);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if resp.clicked() {
        app.open_backstage(crate::backstage::Page::Home);
    }
    resp
}

/// Which tabs the row shows, given each tab's width and whether it must stay (the current tab
/// and contextual ones): all when they fit, else as many as fit in order with room kept for the
/// » button, the pinned ones first.
pub fn fit_tabs(widths: &[(f32, bool)], room: f32, more_w: f32) -> Vec<bool> {
    let w = |x: f32| if x.is_finite() { x.max(0.0) } else { 0.0 };
    if widths.iter().map(|(x, _)| w(*x)).sum::<f32>() <= room + 0.5 {
        return vec![true; widths.len()];
    }
    let mut left = room - more_w - widths.iter().filter(|(_, pinned)| *pinned).map(|(x, _)| w(*x)).sum::<f32>();
    widths
        .iter()
        .map(|(x, pinned)| {
            if *pinned {
                return true;
            }
            if w(*x) <= left + 0.5 {
                left -= w(*x);
                true
            } else {
                left = f32::NEG_INFINITY;
                false
            }
        })
        .collect()
}

/// The ribbon tabs, left aligned: the active one in the accent, bold, underlined; contextual
/// ones in the accent. Tabs that don't fit go into a » menu.
fn tab_row(ui: &mut Ui, app: &mut SheetApp, t: &Tokens, tabs: &[&'static str], height: f32) {
    let lang = app.ui.language;
    let contextual = crate::ribbon::contextual_tabs(app);
    let labels: Vec<&str> = tabs.iter().map(|tab| lang.tr(tab)).collect();
    let widths: Vec<(f32, bool)> = tabs
        .iter()
        .zip(&labels)
        .map(|(tab, shown)| {
            let w = ui.painter().layout_no_wrap(shown.to_string(), strong(13.0), t.text).size().x + 20.0;
            (w, *tab == app.ui.ribbon_tab || contextual.contains(tab))
        })
        .collect();
    let fits = fit_tabs(&widths, ui.available_width(), MORE_TABS_W);
    for (((tab, shown), (w, _)), fit) in tabs.iter().zip(&labels).zip(&widths).zip(&fits) {
        if !fit {
            continue;
        }
        let (r, resp) = ui.allocate_exact_size(vec2(*w, height), Sense::click());
        let active = app.ui.ribbon_tab == *tab;
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, active, *shown));
        if resp.hovered() && !active {
            ui.painter().rect_filled(r.shrink2(vec2(2.0, 6.0)), 6.0, t.hover);
        }
        let color = if active || contextual.contains(tab) { t.accent } else { t.text };
        ui.painter().text(r.center(), Align2::CENTER_CENTER, *shown, if active { strong(13.0) } else { regular(13.0) }, color);
        if active {
            ui.painter().rect_filled(Rect::from_min_max(pos2(r.min.x + 10.0, r.max.y - 2.0), pos2(r.max.x - 10.0, r.max.y)), 1.0, t.accent);
        }
        if resp.double_clicked() {
            app.ui.ribbon_collapsed = !app.ui.ribbon_collapsed;
        } else if resp.clicked() {
            app.ui.ribbon_tab = tab.to_string();
            app.ui.ribbon_collapsed = false;
        }
        if app.keytips.prefix() == Some("")
            && let Some(sequence) = crate::keytips::keytip_table::tab_sequence(tab)
        {
            crate::ribbon::keytip(app, ui, sequence, pos2(r.center().x, r.bottom() - 4.0));
        }
    }
    let left_out: Vec<(&str, &str)> = tabs.iter().zip(&labels).zip(&fits).filter(|(_, fit)| !**fit).map(|((tab, shown), _)| (*tab, *shown)).collect();
    if !left_out.is_empty() {
        let (r, _) = ui.allocate_exact_size(vec2(MORE_TABS_W, height), Sense::hover());
        let resp = ui.interact(r, more_tabs_id(), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r.shrink2(vec2(0.0, 6.0)), 6.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(14.0, 14.0)), Icon::ChevronDoubleRight, t.text_dim);
        let tip = lang.tr("More tabs");
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
        egui::Popup::menu(&resp).show(|ui| {
            for (tab, shown) in left_out {
                if ui.selectable_label(false, shown).clicked() {
                    app.ui.ribbon_tab = tab.to_string();
                    app.ui.ribbon_collapsed = false;
                    ui.close();
                }
            }
        });
    }
}

/// The save state: "Saved" with a check, or "Not saved". A click opens AutoSave, Save and Save As.
fn save_state(ui: &mut Ui, app: &mut SheetApp, t: &Tokens) {
    let lang = app.ui.language;
    let saved = app.session.active().is_some_and(|d| !d.is_dirty() && d.path.is_some());
    let (text, color) = if saved { (lang.tr("Saved"), saved_color(ui.visuals().dark_mode)) } else { (lang.tr("Not saved"), t.text_dim) };
    let font = regular(12.0);
    let tw = ui.painter().layout_no_wrap(text.to_string(), font.clone(), color).size().x;
    let icon_w = if saved { 20.0 } else { 0.0 };
    let (r, _) = ui.allocate_exact_size(vec2(8.0 + icon_w + tw + 4.0 + 10.0 + 8.0, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, save_state_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    if saved {
        icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(15.0, 15.0)), Icon::Check, color);
    }
    let x = r.min.x + 8.0 + icon_w;
    ui.painter().text(pos2(x, r.center().y), Align2::LEFT_CENTER, text, font, color);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(x + tw + 9.0, r.center().y + 1.0), vec2(10.0, 10.0)), Icon::Chevron, t.text_dim);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        let mut on = app.grid.autosave();
        if ui.checkbox(&mut on, lang.tr("AutoSave")).on_hover_text(lang.tr("AutoSave: save after every change (workbooks saved to a file)")).changed()
        {
            app.grid.toggle_autosave();
        }
        ui.separator();
        if ui.button(lang.tr("Save")).clicked() && app.commit_edit(0, 0, false, false) {
            app.run_or_alert("file.save", json!({}));
            ui.close();
        }
        if ui.button(lang.tr("Save As…")).clicked() && app.commit_edit(0, 0, false, false) {
            app.open_dialog("saveAs", json!({}));
            ui.close();
        }
    });
}

/// The search box: a click (or its shortcut) opens Search Commands. Narrow windows show the
/// magnifier alone, so the tabs keep their room.
fn search_box(ui: &mut Ui, app: &mut SheetApp, t: &Tokens, narrow: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(if narrow { CONTROL_H + 4.0 } else { 220.0 }, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, search_id(), Sense::click());
    ui.painter().rect(r, 8.0, if resp.hovered() { t.input_bg } else { t.window }, Stroke::new(1.0, t.ribbon_border), egui::StrokeKind::Inside);
    let icon_x = if narrow { r.center().x } else { r.min.x + 16.0 };
    icons::paint(ui.painter(), Rect::from_center_size(pos2(icon_x, r.center().y), vec2(14.0, 14.0)), Icon::Search, t.text_dim);
    let ph = app.ui.language.tr("Search commands and help");
    if !narrow {
        ui.painter().with_clip_rect(r.shrink2(vec2(8.0, 0.0))).text(
            pos2(r.min.x + 30.0, r.center().y),
            Align2::LEFT_CENTER,
            ph,
            regular(12.0),
            t.text_dim,
        );
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, ph));
    if resp.on_hover_text(app.ui.language.tr("Search commands (⌘⇧Space)")).clicked() {
        app.open_dialog("commandSearch", json!({}));
    }
}

/// Comments: the workbook's comments. Narrow windows show the icon only.
fn comments_button(ui: &mut Ui, app: &mut SheetApp, t: &Tokens, narrow: bool) {
    let label = app.ui.language.tr("Comments");
    let font = regular(12.0);
    let tw = if narrow { 0.0 } else { ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.text).size().x + 6.0 };
    let (r, resp) = ui.allocate_exact_size(vec2(8.0 + 16.0 + tw + 8.0, CONTROL_H), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 16.0, r.center().y), vec2(16.0, 16.0)), Icon::Comment, t.text_dim);
    if !narrow {
        ui.painter().text(pos2(r.min.x + 30.0, r.center().y), Align2::LEFT_CENTER, label, font, t.text);
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if resp.on_hover_text(label).clicked() {
        app.open_dialog("comments", json!({}));
    }
}

/// Share: the accent fill and its text colour; a menu of the copies to share (PDF, workbook, CSV,
/// web page), as upstream's title bar has.
fn share_button(ui: &mut Ui, app: &mut SheetApp, t: &Tokens) {
    let lang = app.ui.language;
    let label = lang.tr("Share");
    let font = strong(12.0);
    let tw = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.on_accent).size().x;
    let (r, resp) = ui.allocate_exact_size(vec2(12.0 + 14.0 + 6.0 + tw + 12.0, CONTROL_H), Sense::click());
    let fill = if resp.hovered() { t.accent_dark } else { t.accent };
    ui.painter().rect_filled(r, 8.0, fill);
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 19.0, r.center().y), vec2(14.0, 14.0)), Icon::Share, t.on_accent);
    ui.painter().text(pos2(r.min.x + 32.0, r.center().y), Align2::LEFT_CENTER, label, font, t.on_accent);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    let resp = resp.on_hover_text(lang.tr("Share: export or save a copy"));
    egui::Popup::menu(&resp).show(|ui| {
        if ui.button(lang.tr("Export as PDF…")).clicked() {
            app.run_or_alert("file.exportPdf", json!({}));
        }
        for (label, fmt) in [("Save a Copy as Excel Workbook (.xlsx)…", "xlsx"), ("Export as CSV…", "csv"), ("Export as Web Page (.html)…", "html")]
        {
            if ui.button(lang.tr(label)).clicked() {
                app.open_dialog("saveCopy", json!({"format": fmt}));
            }
        }
    });
}

/// "⋯": the light/dark theme, Options and About.
fn more_menu(ui: &mut Ui, app: &mut SheetApp, t: &Tokens) {
    let (r, _) = ui.allocate_exact_size(vec2(CONTROL_H, CONTROL_H), Sense::hover());
    let resp = ui.interact(r, more_button_id(), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, 6.0, t.hover);
    }
    icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(18.0, 18.0)), Icon::More, t.text_dim);
    let lang = app.ui.language;
    let tip = lang.tr("More options");
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    let resp = resp.on_hover_text(tip);
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        crate::ribbon::theme_choices(app, ui);
        ui.separator();
        if ui.button(lang.tr("Options")).clicked() {
            app.open_backstage(crate::backstage::Page::Options);
            ui.close();
        }
        if ui.button(lang.tr("About GridCraft")).clicked() {
            app.open_dialog("about", json!({}));
            ui.close();
        }
    });
}

/// The command strip: the current tab's groups, then "…" with the ones that didn't fit.
fn strip(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    egui::Panel::top("ribbon")
        .exact_size(STRIP_H)
        .frame(
            egui::Frame::NONE
                .fill(t.ribbon)
                .inner_margin(egui::Margin { left: 12, right: 8, top: 3, bottom: 3 })
                .stroke(Stroke::new(1.0, t.ribbon_border)),
        )
        .show(ui, |ui| {
            let key = Id::new("nexus_wps_group_widths").with(app.ui.ribbon_tab.as_str());
            let known: Vec<f32> = ui.data(|d| d.get_temp(key)).unwrap_or_default();
            let full = ui.available_rect_before_wrap();
            let room = (full.width() - OVERFLOW_W).max(0.0);
            let shown = fit_groups(&known, room);
            let content = Rect::from_min_size(full.min, vec2(room, full.height()));
            // Salted with the tab: every tab's controls keep their own state (a stack's rows).
            let builder = egui::UiBuilder::new().id_salt(("wps_strip", app.ui.ribbon_tab.as_str())).max_rect(content);
            let mut row = ui.new_child(builder.layout(egui::Layout::left_to_right(egui::Align::Min)));
            row.set_clip_rect(content.intersect(ui.clip_rect()));
            row.spacing_mut().item_spacing = vec2(2.0, 2.0);
            let gate = with_gate(ui.ctx(), Mode::Strip { shown }, || crate::ribbon::tab_groups(app, &mut row));
            let mut widths = known;
            widths.resize(gate.next, 0.0);
            for (w, m) in widths.iter_mut().zip(&gate.widths) {
                if let Some(m) = m {
                    *w = *m;
                }
            }
            if ui.data(|d| d.get_temp::<Vec<f32>>(key)).as_ref() != Some(&widths) {
                ui.data_mut(|d| d.insert_temp(key, widths));
                ui.ctx().request_repaint();
            }
            if shown >= gate.next {
                return;
            }
            let r = Rect::from_min_size(pos2(full.max.x - OVERFLOW_W + 4.0, full.min.y), vec2(OVERFLOW_W - 4.0, widgets::COMPACT_H));
            let resp = ui.interact(r, overflow_button_id(), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(r, 6.0, t.hover);
            }
            icons::paint(ui.painter(), Rect::from_center_size(r.center(), vec2(20.0, 20.0)), Icon::More, t.text_dim);
            let tip = app.ui.language.tr("More commands");
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
            let resp = resp.on_hover_text(tip);
            overflow_panel(app, ui, &resp, shown);
        });
}

/// Where the "…" panel was drawn last frame, while it is open: (its tab, its rect).
fn overflow_open_id() -> Id {
    Id::new("nexus_wps_overflow_open")
}

/// The "…" panel: the groups from `from` on, under the button. Not an egui popup — only one of
/// those can be open, and the groups' own menus (Borders, Sort & Filter…) are popups that must
/// open over it. It closes on Escape, on another tab, or on a click outside it (unless that click
/// was in one of the groups' menus).
fn overflow_panel(app: &mut SheetApp, ui: &mut Ui, button: &Response, from: usize) {
    let ctx = ui.ctx().clone();
    let mut open: Option<(String, Rect)> = ctx.data(|d| d.get_temp(overflow_open_id()));
    if button.clicked() {
        open = match open {
            Some(_) => None,
            None => Some((app.ui.ribbon_tab.clone(), Rect::NOTHING)),
        };
    } else if let Some((tab, rect)) = &open {
        let outside =
            ctx.input(|i| i.pointer.primary_pressed() && i.pointer.interact_pos().is_some_and(|p| !rect.contains(p) && !button.rect.contains(p)));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if *tab != app.ui.ribbon_tab || escape || (outside && !egui::Popup::is_any_open(&ctx)) {
            open = None;
        }
    }
    let Some((tab, _)) = open else {
        ctx.data_mut(|d| d.remove::<(String, Rect)>(overflow_open_id()));
        return;
    };
    let area = egui::Area::new(overflow_open_id().with("area"))
        .order(egui::Order::Foreground)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(button.rect.right_bottom() + vec2(0.0, 6.0))
        .constrain(true)
        .show(&ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                let builder = egui::UiBuilder::new().id_salt(("wps_overflow", tab.as_str())).layout(egui::Layout::left_to_right(egui::Align::Min));
                ui.scope_builder(builder, |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                    with_gate(&ctx, Mode::Menu { from }, || crate::ribbon::tab_groups(app, ui));
                });
            });
        });
    let rect = area.response.rect;
    ctx.data_mut(|d| d.insert_temp(overflow_open_id(), (tab, rect)));
}

/// Whether the "…" panel is open.
pub fn overflow_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<(String, Rect)>(overflow_open_id())).is_some()
}

/// How many groups, from the first, fit in `room` points given their widths last drawn: all of
/// them (`usize::MAX`) when they fit. A group not measured yet counts as fitting, so it is drawn
/// (clipped at worst) and measured once.
pub fn fit_groups(widths: &[f32], room: f32) -> usize {
    let mut x = 0.0;
    for (i, w) in widths.iter().enumerate() {
        x += if w.is_finite() { w.max(0.0) } else { 0.0 };
        if x > room + 0.5 {
            return i;
        }
    }
    usize::MAX
}

/// Which groups a pass over the tab draws.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    /// The strip: the first `shown` groups.
    Strip { shown: usize },
    /// The "…" menu: the groups from `from` on.
    Menu { from: usize },
}

/// A pass over the tab's groups: the mode, how many groups were met, and the width of each the
/// strip drew.
#[derive(Clone, Debug)]
struct Gate {
    mode: Mode,
    next: usize,
    widths: Vec<Option<f32>>,
    /// A stack in the group being drawn is still finding its layout, so the group's width this
    /// frame isn't its width: it is recorded as unknown (and the group drawn again).
    unsettled: bool,
}

fn gate_id() -> Id {
    Id::new("nexus_wps_gate")
}

/// Run `draw` with groups gated by `mode`; returns what the pass met.
fn with_gate(ctx: &egui::Context, mode: Mode, draw: impl FnOnce()) -> Gate {
    ctx.data_mut(|d| {
        d.insert_temp(gate_id(), Gate { mode, next: 0, widths: Vec::new(), unsettled: false });
        d.insert_temp(compact_id(), true);
    });
    draw();
    let gate = ctx.data_mut(|d| {
        d.remove::<bool>(compact_id());
        let gate = d.get_temp::<Gate>(gate_id());
        d.remove::<Gate>(gate_id());
        gate
    });
    gate.unwrap_or(Gate { mode, next: 0, widths: Vec::new(), unsettled: false })
}

fn compact_id() -> Id {
    Id::new("nexus_wps_compact")
}

/// Whether the ribbon being drawn is the WPS strip (shorter: see [`widgets::COMPACT_H`]).
pub(crate) fn compact(ui: &Ui) -> bool {
    ui.data(|d| d.get_temp::<bool>(compact_id())).unwrap_or(false)
}

#[cfg(test)]
thread_local! {
    /// Groups drawn taller than the strip's two rows (test builds only): "title: height".
    pub(crate) static TALL_GROUPS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A ribbon group in the WPS strip or its "…" panel: drawn when this pass draws it, then a
/// divider; the strip records its width so the groups that don't fit can fold away.
pub(crate) fn group(ui: &mut Ui, app: &mut SheetApp, title: &str, add: impl FnOnce(&mut Ui, &mut SheetApp)) {
    let Some(slot) = group_begin(ui) else { return };
    let t = Tokens::get(ui.ctx());
    let h = widgets::COMPACT_H;
    let start = ui.cursor().min;
    let content = ui.allocate_ui_with_layout(vec2(ui.available_width(), h), egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
        add(ui, app);
    });
    #[cfg(test)]
    if content.response.rect.height() > h + 0.5 {
        TALL_GROUPS.with(|g| g.borrow_mut().push(format!("{title}: {:.1}", content.response.rect.height())));
    }
    #[cfg(not(test))]
    let _ = (content, title);
    ui.add_space(5.0);
    let x = ui.cursor().min.x;
    ui.painter().line_segment([pos2(x, start.y + 6.0), pos2(x, start.y + h - 6.0)], Stroke::new(1.0, t.ribbon_border));
    ui.add_space(6.0);
    group_end(ui, slot, ui.cursor().min.x - start.x);
}

/// A stack of controls (upstream draws them one under another) in the WPS strip. Two rows fit
/// as they are (the Font group's two rows). Three or more flow left to right in two rows (Cut and
/// Copy, then Format Painter), as narrow as two rows allow. What fits is found over a few frames
/// — one column, then one row, then from half that width up until the controls take two rows —
/// and kept per interface language.
pub(crate) fn compact_stack(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let key = ui.id().with(("wps_stack", ui.next_auto_id(), crate::i18n::current(ui.ctx()).code()));
    let state: Option<StackFit> = ui.data(|d| d.get_temp(key));
    // Two rows run a point or two past the strip's content height at most; three can't fit.
    let two_rows = widgets::COMPACT_H + 8.0;
    let (layout, max) = match state {
        None | Some(StackFit::Column) => (egui::Layout::top_down(egui::Align::Min), None),
        Some(StackFit::MeasureRow) => (egui::Layout::left_to_right(egui::Align::Min), None),
        Some(StackFit::Rows { width, .. }) => (
            egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
            Some(Rect::from_min_size(ui.cursor().min, vec2(width.max(1.0), widgets::COMPACT_H))),
        ),
    };
    let mut builder = egui::UiBuilder::new().layout(layout);
    if let Some(max) = max {
        builder = builder.max_rect(max);
    }
    let r = ui.scope_builder(builder, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 2.0);
        add(ui);
    });
    let size = r.response.rect.size();
    if !size.x.is_finite() || !size.y.is_finite() {
        return;
    }
    let next = match state {
        None if size.y <= two_rows => Some(StackFit::Column),
        None => Some(StackFit::MeasureRow),
        Some(StackFit::MeasureRow) => Some(StackFit::Rows { total: size.x, width: (size.x / 2.0).ceil() }),
        // (A point past one row's width, so rounding never wraps its last control.)
        Some(StackFit::Rows { total, width }) if size.y > two_rows && width < total + 1.0 => {
            Some(StackFit::Rows { total, width: (width + (total * 0.15).max(12.0)).min(total + 1.0) })
        }
        _ => None,
    };
    if let Some(next) = next {
        ui.data_mut(|d| {
            d.insert_temp(key, next);
            if let Some(mut gate) = d.get_temp::<Gate>(gate_id()) {
                gate.unsettled = true;
                d.insert_temp(gate_id(), gate);
            }
        });
        ui.ctx().request_repaint();
    }
}

/// How a [`compact_stack`] lays its controls out.
#[derive(Clone, Copy, Debug, PartialEq)]
enum StackFit {
    /// One under another (they fit in two rows).
    Column,
    /// Being measured in one row.
    MeasureRow,
    /// Flowing in rows `width` wide; `total` is their width in one row.
    Rows { total: f32, width: f32 },
}

/// Where a group is being drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    /// The WPS strip, as its group number `n`.
    Strip(usize),
    /// The WPS "…" menu, as its group number `n`.
    Menu(usize),
}

/// Called as a ribbon group starts: where it is drawn, or `None` when this pass leaves it out.
fn group_begin(ui: &Ui) -> Option<Slot> {
    ui.ctx().data_mut(|d| {
        let mut gate = d.get_temp::<Gate>(gate_id())?;
        let n = gate.next;
        gate.next = n.saturating_add(1);
        gate.unsettled = false;
        d.insert_temp(gate_id(), gate.clone());
        match gate.mode {
            Mode::Strip { shown } if n < shown => Some(Slot::Strip(n)),
            Mode::Menu { from } if n >= from => Some(Slot::Menu(n)),
            _ => None,
        }
    })
}

/// Called as a ribbon group ends, with the width it took (its divider included).
fn group_end(ui: &Ui, slot: Slot, width: f32) {
    let Slot::Strip(n) = slot else { return };
    ui.ctx().data_mut(|d| {
        let Some(mut gate) = d.get_temp::<Gate>(gate_id()) else { return };
        if gate.widths.len() <= n {
            gate.widths.resize(n.saturating_add(1), None);
        }
        let unsettled = std::mem::take(&mut gate.unsettled);
        if let Some(w) = gate.widths.get_mut(n) {
            // 0 = not known: the strip draws it (clipped at worst) and measures it again.
            *w = Some(if unsettled { 0.0 } else { width });
        }
        d.insert_temp(gate_id(), gate);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use gridcraft_engine::Session;

    type H = egui_kittest::Harness<'static, SheetApp>;

    fn session(sample: Option<&str>) -> Session {
        let mut s = Session::new();
        match sample {
            Some(name) => {
                s.execute("file.new", serde_json::json!({"sample": name})).unwrap();
            }
            None => {
                s.new_workbook();
            }
        }
        s
    }

    /// A workbook in the WPS layout, embedded as a tab of the NexusOffice window.
    fn harness(size: (f32, f32), lang: crate::i18n::Language, s: Session, setup: impl FnOnce(&mut SheetApp)) -> H {
        let mut a = SheetApp::new(s, crate::Services::default());
        a.wps = true;
        a.embedded = true;
        a.ui.language = lang;
        setup(&mut a);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(size.0, size.1)).with_max_steps(64).build_ui_state(
            |ui, app: &mut SheetApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        // Stacks and the "…" fold settle over a few frames.
        h.run_steps(16);
        h
    }

    fn click_id(h: &mut H, id: Id) {
        let r = h.ctx.read_response(id).map(|r| r.rect);
        let r = r.unwrap_or_else(|| panic!("{id:?} is not drawn"));
        h.input_mut().events.push(egui::Event::PointerMoved(r.center()));
        h.run_steps(1);
        for pressed in [true, false] {
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: r.center(),
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.run_steps(1);
        }
        h.run_steps(6);
    }

    fn cell(app: &SheetApp, a1: &str) -> String {
        let at = gridcraft_engine::core::CellRef::parse(a1).unwrap();
        app.session.active().and_then(|d| d.wb.active()).map(|sh| sh.value(at).display()).unwrap_or_default()
    }

    /// With the WPS layout there is no title bar (upstream's holds the AutoSave switch); File (Tệp)
    /// opens Backstage.
    #[test]
    fn no_title_bar_and_file_opens_backstage() {
        use crate::i18n::Language;
        let up = harness((1440.0, 900.0), Language::En, session(None), |a| a.wps = false);
        assert!(up.query_by_label("AutoSave").is_some(), "the title bar's AutoSave switch is found when it is there");

        let mut h = harness((1440.0, 900.0), Language::Vi, session(None), |_| {});
        assert!(h.query_by_label("AutoSave").is_none() && h.query_by_label("Tự động lưu").is_none(), "no title bar");
        h.get_by_label("Tệp").click();
        h.run_steps(4);
        assert!(h.state().backstage.is_some(), "File opens Backstage");
    }

    /// The Quick Access Toolbar keeps working in the top row: Undo and Redo, and Save writes the
    /// workbook; the save state then says so, and its menu holds AutoSave.
    #[test]
    fn quick_access_save_undo_and_redo_work() {
        let path = std::env::temp_dir().join(format!("nexus-sheet-wps-qat-{}.xlsx", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut h = harness((1440.0, 900.0), crate::i18n::Language::En, session(None), |a| {
            a.run("cell.set", serde_json::json!({"cell": "A1", "input": "Xin chào"})).unwrap();
        });
        assert_eq!(cell(h.state(), "A1"), "Xin chào");
        h.get_by_label(&crate::i18n::brand("Undo (⌘Z)")).click();
        h.run_steps(4);
        assert_eq!(cell(h.state(), "A1"), "", "Undo undid the typing");
        h.get_by_label(&crate::i18n::brand("Redo (⌘Y)")).click();
        h.run_steps(4);
        assert_eq!(cell(h.state(), "A1"), "Xin chào", "Redo redid it");
        assert!(h.query_by_label("Not saved").is_some());
        let p = path.to_string_lossy().into_owned();
        h.state_mut().session.active_mut().unwrap().path = Some(p);
        h.run_steps(2);
        h.get_by_label(&crate::i18n::brand("Save (⌘S)")).click();
        h.run_steps(6);
        assert!(path.exists(), "Save wrote the file");
        assert!(!h.state().session.active().unwrap().is_dirty());
        assert!(h.query_by_label("Saved").is_some(), "the save state follows");
        click_id(&mut h, save_state_id());
        assert!(h.query_by_label("AutoSave").is_some(), "AutoSave is in the save state's menu");
        let _ = std::fs::remove_file(&path);
    }

    /// Every tab, the contextual Table Design and Chart Design included, draws its strip at both
    /// window sizes without panicking, and no group is taller than the strip's two rows.
    #[test]
    fn every_tab_fits_the_strip_at_both_sizes() {
        for size in [(1440.0, 900.0), (1024.0, 700.0)] {
            let mut h = harness(size, crate::i18n::Language::Vi, session(Some("sales")), |a| {
                // Inside the sample's table, with its first chart selected.
                a.run("selection.set", serde_json::json!({"range": "B5"})).unwrap();
                a.selected_chart = a.session.active().and_then(|d| d.wb.active()).and_then(|sh| sh.charts.first()).map(|c| c.id);
            });
            let tabs = crate::ribbon::shown_tabs(h.state());
            assert!(tabs.contains(&"Table Design") && tabs.contains(&"Chart Design"), "{tabs:?}");
            for tab in tabs {
                h.state_mut().ui.ribbon_tab = tab.to_string();
                // The first look at a tab settles its stacks' rows over a few frames.
                h.run_steps(30);
                TALL_GROUPS.with(|g| g.borrow_mut().clear());
                h.run_steps(1);
                let tall: Vec<String> = TALL_GROUPS.with(|g| g.take());
                assert!(tall.is_empty(), "{tab} at {size:?}: {tall:?}");
                assert_eq!(h.state().ui.ribbon_tab, tab);
            }
        }
    }

    /// At 1024 pt Home's last groups fold into "…": its panel shows them and their commands run.
    #[test]
    fn narrow_windows_fold_groups_into_the_overflow_panel() {
        let fill = |a: &mut SheetApp| {
            for (c, v) in [("A1", "cam"), ("A2", "anh"), ("A3", "bưởi")] {
                a.run("cell.set", serde_json::json!({"cell": c, "input": v})).unwrap();
            }
            a.run("selection.set", serde_json::json!({"range": "A1:A3"})).unwrap();
        };
        let wide = harness((1920.0, 900.0), crate::i18n::Language::En, session(None), fill);
        assert!(wide.ctx.read_response(overflow_button_id()).is_none(), "everything fits a wide window");

        let mut h = harness((1024.0, 700.0), crate::i18n::Language::En, session(None), fill);
        assert!(h.query_by_label("Sort & Filter").is_none(), "Editing folded away");
        click_id(&mut h, overflow_button_id());
        assert!(overflow_open(&h.ctx));
        h.get_by_label("Sort & Filter").click();
        h.run_steps(4);
        h.get_by_label("Sort A to Z").click();
        h.run_steps(4);
        assert_eq!(cell(h.state(), "A1"), "anh", "the folded group's command ran");
        h.key_press(egui::Key::Escape);
        h.run_steps(4);
        assert!(!overflow_open(&h.ctx), "Escape closes the panel");
    }

    /// Search, Comments and Share stay in the top row; tabs switch, and the » holds the rest.
    #[test]
    fn search_comments_share_and_tabs_are_reachable() {
        let mut h = harness((1440.0, 900.0), crate::i18n::Language::Vi, session(None), |_| {});
        click_id(&mut h, search_id());
        assert!(h.state().dialog.as_ref().is_some_and(|d| d.name == "commandSearch"), "the box opens Search");
        h.state_mut().dialog = None;
        h.run_steps(2);
        h.get_by_label("Dữ liệu").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.ribbon_tab, "Data");
        h.get_by_label("Chia sẻ").click();
        h.run_steps(4);
        assert!(h.query_by_label("Xuất PDF…").is_some(), "Share offers the copies to share");
        h.key_press(egui::Key::Escape);
        h.run_steps(2);
        // A narrow window keeps every tab reachable through ».
        let mut n = harness((1024.0, 700.0), crate::i18n::Language::Vi, session(None), |_| {});
        assert!(n.query_by_label("Tự động hóa").is_none(), "Automate doesn't fit");
        click_id(&mut n, more_tabs_id());
        n.get_by_label("Tự động hóa").click();
        n.run_steps(4);
        assert_eq!(n.state().ui.ribbon_tab, "Automate");
    }

    /// Embedded, the app leaves the host window's title alone; stand-alone it sets it.
    #[test]
    fn embedded_leaves_the_window_title_alone() {
        let alone = harness((1024.0, 700.0), crate::i18n::Language::En, session(None), |a| a.embedded = false);
        assert!(alone.state().grid.last_title.is_some(), "the stand-alone app names the window");
        let hosted = harness((1024.0, 700.0), crate::i18n::Language::En, session(None), |_| {});
        assert!(hosted.state().grid.last_title.is_none(), "a hosted app never sets the window title");
    }

    #[test]
    fn groups_and_tabs_fit_from_the_first_and_junk_widths_are_harmless() {
        assert_eq!(fit_groups(&[], 100.0), usize::MAX);
        assert_eq!(fit_groups(&[40.0, 40.0], 100.0), usize::MAX);
        assert_eq!(fit_groups(&[40.0, 40.0, 40.0], 100.0), 2);
        assert_eq!(fit_groups(&[120.0, 10.0], 100.0), 0);
        assert_eq!(fit_groups(&[40.0, 0.0, 40.0], 100.0), usize::MAX, "0 = not measured yet");
        assert_eq!(fit_groups(&[f32::NAN, f32::INFINITY, 40.0], 100.0), usize::MAX, "junk widths count as 0");
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false)], 100.0, 20.0), [true, true]);
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, false)], 100.0, 20.0), [true, true, false]);
        // The current tab stays even when it comes last.
        assert_eq!(fit_tabs(&[(40.0, false), (40.0, false), (40.0, true)], 100.0, 20.0), [true, false, true]);
        assert_eq!(fit_tabs(&[(f32::NAN, false), (f32::INFINITY, true)], 10.0, 20.0), [true, true]);
    }

    /// File's label keeps 4.5:1 on its faint accent fill, in both themes and with a chosen accent;
    /// so does Share's text on the accent and "Saved" on the row.
    #[test]
    fn file_label_is_readable_on_its_fill() {
        let accents = [
            None,
            Some(Color32::from_rgb(0x4F, 0x46, 0xE5)),
            Some(Color32::from_rgb(0xE1, 0x1D, 0x48)),
            Some(Color32::from_rgb(0xFA, 0xCC, 0x15)),
            Some(Color32::from_rgb(0xB4, 0x53, 0x09)),
        ];
        for base in [Tokens::light(), Tokens::dark()] {
            for accent in accents {
                let t = accent.map_or(base, |c| theme::with_accent(base, c));
                for hovered in [false, true] {
                    let fill = file_fill(&t, hovered);
                    assert_ne!(fill, t.ribbon, "File keeps its tint");
                    let ratio = theme::contrast_ratio(file_ink(&t, fill), fill);
                    assert!(ratio >= 4.5, "{accent:?} hovered {hovered}: {ratio}");
                }
                assert!(theme::contrast_ratio(t.on_accent, t.accent) >= 4.5, "Share's text on the accent");
            }
        }
        for dark in [false, true] {
            let row = if dark { Tokens::dark().ribbon } else { Tokens::light().ribbon };
            assert!(theme::contrast_ratio(saved_color(dark), row) >= 4.5, "Saved, dark {dark}");
        }
    }
}
