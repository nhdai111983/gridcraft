//! The NexusOffice layout (Henry, 2026-10-23; ROADMAP F2.2): a menu bar and one row of tools in
//! place of the ribbon, like the WPS-style layout Nexus Docs has. The menus are built from the
//! command registry (every command under the ribbon tab and group its location names), so nothing
//! the ribbon offers is lost; the tool row holds the everyday spreadsheet tools, and the tools that
//! don't fit a narrow window move into a "More tools" menu at its end. The ribbon's code stays as
//! upstream wrote it and draws when [`SheetApp::toolbar`] is off (upstream's own tests and builds).

use egui::{Align2, Rect, Sense, Stroke, Ui, pos2, vec2};
use gridcraft_engine::CommandSpec;
use gridcraft_engine::model::HAlign;
use serde_json::json;

use crate::SheetApp;
use crate::icons::Icon;
use crate::ribbon::{act, active_style, font_combo, menu_items, size_combo, theme_colors};
use crate::theme::{self, Tokens};
use crate::widgets::{color_palette, icon_button, small_button, split_button, toggle_button};

/// Height of the menu bar and of the tool row.
pub const MENU_H: f32 = 28.0;
pub const TOOLS_H: f32 = 36.0;
/// The menu for PivotTable commands (Excel's two PivotTable tabs), shown in a pivot report.
pub const PIVOT: &str = "PivotTable";

/// One entry of a menu group: a command, or a submenu (the location's third part, e.g.
/// Home › Cells › Insert) of commands.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Command(&'static str),
    Sub(&'static str, Vec<&'static str>),
}

/// One menu of the menu bar: a ribbon tab and its commands, grouped by ribbon group (in registry
/// order) as `(group, entries)`; `""` is a command placed on the tab itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub tab: &'static str,
    pub groups: Vec<(&'static str, Vec<Entry>)>,
}

/// The menu a location's first part goes to. A few registry locations name tabs the ribbon shows
/// under another name; they join that menu so their commands stay reachable.
pub fn menu_tab(first: &'static str) -> &'static str {
    match first {
        "Edit" => "Home",
        "Window" => "View",
        "Table" => "Table Design",
        "Format" => "Chart Design",
        "PivotTable Analyze" | "PivotTable Design" => PIVOT,
        other => other,
    }
}

/// The menus for `tabs` (File excluded: it opens the File page): every command whose location
/// starts with the tab (after [`menu_tab`]) goes into that tab's menu, under its group. Tabs with no
/// command are left out.
pub fn menus(commands: &[CommandSpec], tabs: &[&str]) -> Vec<Menu> {
    let mut out = Vec::new();
    for &tab in tabs.iter().filter(|t| **t != "File") {
        let mut groups: Vec<(&'static str, Vec<Entry>)> = Vec::new();
        let mut name: Option<&'static str> = None;
        for c in commands {
            let Some(first) = c.menu.first().copied() else { continue };
            if menu_tab(first) != tab {
                continue;
            }
            name.get_or_insert(menu_tab(first));
            let group = c.menu.get(1).copied().unwrap_or("");
            let entries = match groups.iter().position(|(g, _)| *g == group) {
                Some(i) => &mut groups[i].1,
                None => {
                    groups.push((group, Vec::new()));
                    let last = groups.len() - 1;
                    &mut groups[last].1
                }
            };
            match c.menu.get(2).copied() {
                None => entries.push(Entry::Command(c.id)),
                Some(sub) => match entries.iter_mut().find(|e| matches!(e, Entry::Sub(s, _) if *s == sub)) {
                    Some(Entry::Sub(_, ids)) => ids.push(c.id),
                    _ => entries.push(Entry::Sub(sub, vec![c.id])),
                },
            }
        }
        if let Some(tab) = name {
            out.push(Menu { tab, groups });
        }
    }
    out
}

/// The tabs whose menus the bar shows: the ribbon's tabs, then the contextual ones that apply now
/// (Table Design in a table, Chart Design with a chart selected, PivotTable in a pivot report).
fn tab_names(app: &SheetApp) -> Vec<&'static str> {
    let mut tabs: Vec<&'static str> = crate::ribbon::TABS.to_vec();
    tabs.extend(crate::ribbon::contextual_tabs(app));
    if crate::pivot_pane::active_pivot(app).is_some() {
        tabs.push(PIVOT);
    }
    tabs
}

/// The menu bar, the tool row and, while a header or footer is edited, its tools.
pub fn show(app: &mut SheetApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("nexus_menu")
        .exact_size(MENU_H)
        .frame(egui::Frame::NONE.fill(t.window).inner_margin(egui::Margin { left: 6, right: 10, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            menu_bar(app, ui);
            brand_line(ui);
        });
    if app.ui.ribbon_collapsed {
        return;
    }
    egui::Panel::top("nexus_tools")
        .exact_size(TOOLS_H)
        .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left: 8, right: 8, top: 0, bottom: 0 }))
        .show(ui, |ui| {
            let r = ui.max_rect();
            ui.painter().line_segment([pos2(r.left() - 8.0, r.bottom()), pos2(r.right() + 8.0, r.bottom())], Stroke::new(1.0, t.ribbon_border));
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
                tools(app, ui);
            });
        });
    // Header & Footer has tools of its own (fields, options) that are no command of the registry.
    if crate::ribbon::contextual_tabs(app).contains(&crate::paper_view::TAB) {
        egui::Panel::top("nexus_hf_tools")
            .exact_size(92.0)
            .frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left: 10, right: 10, top: 4, bottom: 4 }))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                        crate::paper_view::ribbon_tab(app, ui);
                    });
                });
            });
    }
}

fn menu_bar(app: &mut SheetApp, ui: &mut Ui) {
    let lang = app.ui.language;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
        // File opens the File page (Backstage), as the ribbon's File tab does.
        if menu_title(ui, lang.tr("File"), false).clicked() {
            app.open_backstage(crate::backstage::Page::Home);
        }
        let tabs = tab_names(app);
        let contextual = crate::ribbon::contextual_tabs(app);
        for m in menus(gridcraft_engine::command_specs(), &tabs) {
            let ctx_tab = contextual.contains(&m.tab) || m.tab == PIVOT;
            let resp = menu_title(ui, lang.tr(m.tab), ctx_tab);
            egui::Popup::menu(&resp).show(|ui| {
                ui.set_min_width(220.0);
                menu_body(app, ui, &m);
            });
        }
    });
}

/// The SonnyTech gradient (indigo → cyan → emerald), 2 pt under the menu bar.
fn brand_line(ui: &mut Ui) {
    let r = ui.max_rect();
    let line = Rect::from_min_max(pos2(r.min.x - 6.0, r.max.y - 2.0), pos2(r.max.x + 10.0, r.max.y));
    let [a, b, c] = theme::BRAND_GRADIENT;
    let mid = line.center().x;
    let mut mesh = egui::Mesh::default();
    for (x0, x1, c0, c1) in [(line.min.x, mid, a, b), (mid, line.max.x, b, c)] {
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(pos2(x0, line.min.y), c0);
        mesh.colored_vertex(pos2(x1, line.min.y), c1);
        mesh.colored_vertex(pos2(x1, line.max.y), c1);
        mesh.colored_vertex(pos2(x0, line.max.y), c0);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}

/// A menu bar title: text that highlights on hover; contextual menus (Table Design…) in the
/// accent colour, as the ribbon draws contextual tabs.
fn menu_title(ui: &mut Ui, text: &str, contextual: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let font = theme::ui_font(13.0);
    let w = ui.painter().layout_no_wrap(text.to_string(), font.clone(), t.text).size().x + 16.0;
    let (r, resp) = ui.allocate_exact_size(vec2(w, MENU_H - 6.0), Sense::click());
    if resp.hovered() || egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&resp)) {
        ui.painter().rect_filled(r, 4.0, t.hover);
    }
    ui.painter().text(r.center(), Align2::CENTER_CENTER, text, font, if contextual { t.accent } else { t.text });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text));
    resp
}

/// A menu's items: one submenu per ribbon group (a menu with a single group lists it directly).
fn menu_body(app: &mut SheetApp, ui: &mut Ui, m: &Menu) {
    let lang = app.ui.language;
    let single = m.groups.len() == 1;
    for (group, entries) in &m.groups {
        if single || group.is_empty() {
            entry_items(app, ui, entries);
        } else {
            ui.menu_button(lang.tr(group), |ui| {
                ui.set_min_width(220.0);
                entry_items(app, ui, entries);
            });
        }
    }
}

fn entry_items(app: &mut SheetApp, ui: &mut Ui, entries: &[Entry]) {
    let lang = app.ui.language;
    for e in entries {
        match e {
            Entry::Command(id) => item(app, ui, id),
            Entry::Sub(name, ids) => {
                ui.menu_button(lang.tr(name), |ui| {
                    ui.set_min_width(200.0);
                    for id in ids {
                        item(app, ui, id);
                    }
                });
            }
        }
    }
}

/// A command as a menu item: its label, its shortcut, greyed out when the command says it can't
/// run now. A click runs it as a menu does (a command that needs settings asks for them).
fn item(app: &mut SheetApp, ui: &mut Ui, id: &str) {
    let Some(c) = gridcraft_engine::find_command(id) else { return };
    let on = (c.enabled)(&app.session).is_ok();
    let label = app.ui.language.tr(c.label).to_string();
    // An empty shortcut would leave a trailing space in the item's accessible name.
    let button = match c.shortcut {
        Some(sc) if !sc.is_empty() => egui::Button::new(label).shortcut_text(sc),
        _ => egui::Button::new(label),
    };
    if ui.add_enabled(on, button).clicked() {
        run_menu_command(app, ui.ctx(), id);
        ui.close();
    }
}

/// Runs a command picked from a menu: Cut, Copy and Paste go through the system clipboard, as the
/// ribbon's buttons do.
fn run_menu_command(app: &mut SheetApp, ctx: &egui::Context, id: &str) {
    match id {
        "edit.copy" | "edit.cut" => app.copy_to_clipboard(ctx, id),
        "edit.paste" => app.paste_from_clipboard(),
        _ => app.run_or_alert(id, json!({})),
    }
}

/// The tool row's clusters, in the order they give way when the window is narrow (the last first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cluster {
    Clipboard,
    Font,
    Format,
    Align,
    Number,
    SortFilter,
    Cells,
    Editing,
}

impl Cluster {
    const ALL: [Cluster; 8] =
        [Cluster::Clipboard, Cluster::Font, Cluster::Format, Cluster::Align, Cluster::Number, Cluster::SortFilter, Cluster::Cells, Cluster::Editing];

    /// Width before the cluster has been drawn once (then its measured width is used).
    fn estimate(self) -> f32 {
        match self {
            Cluster::Clipboard => 132.0,
            Cluster::Font => 224.0,
            Cluster::Format => 220.0,
            Cluster::Align => 170.0,
            Cluster::Number => 156.0,
            Cluster::SortFilter => 96.0,
            Cluster::Cells => 90.0,
            Cluster::Editing => 84.0,
        }
    }

    /// The cluster's commands as menu items, for the More tools menu when it doesn't fit.
    fn items(self, app: &SheetApp) -> Vec<(&'static str, &'static str, serde_json::Value)> {
        match self {
            Cluster::Clipboard => vec![("Paste", "edit.paste", json!({})), ("Format Painter", "edit.formatPainter", json!({}))],
            Cluster::Font => vec![
                ("Font…", "dialog:formatCells", json!({"tab": "Font"})),
                ("Increase Font Size", "home.increaseFontSize", json!({})),
                ("Decrease Font Size", "home.decreaseFontSize", json!({})),
            ],
            Cluster::Format => vec![
                ("Bold", "home.bold", json!({})),
                ("Italic", "home.italic", json!({})),
                ("Underline", "home.underline", json!({})),
                ("Borders", "home.borders", json!({"preset": app.grid.last_border.clone()})),
                ("Fill Color", "home.fillColor", json!({"color": app.grid.last_fill.clone()})),
                ("Font Color", "home.fontColor", json!({"color": app.grid.last_font_color.clone()})),
            ],
            Cluster::Align => vec![
                ("Align Left", "home.alignLeft", json!({})),
                ("Center", "home.alignCenter", json!({})),
                ("Align Right", "home.alignRight", json!({})),
                ("Wrap Text", "home.wrapText", json!({})),
                ("Merge & Center", "home.mergeCenter", json!({})),
            ],
            Cluster::Number => vec![
                ("Accounting Number Format", "home.accounting", json!({})),
                ("Percent Style", "home.percent", json!({})),
                ("Comma Style", "home.comma", json!({})),
                ("Increase Decimal", "home.increaseDecimal", json!({})),
                ("Decrease Decimal", "home.decreaseDecimal", json!({})),
            ],
            Cluster::SortFilter => vec![
                ("Sort A to Z", "data.sortAscending", json!({})),
                ("Sort Z to A", "data.sortDescending", json!({})),
                ("Filter", "data.filter", json!({})),
            ],
            Cluster::Cells => vec![
                ("Insert Sheet Rows", "home.insertRows", json!({})),
                ("Insert Sheet Columns", "home.insertColumns", json!({})),
                ("Delete Sheet Rows", "home.deleteRows", json!({})),
                ("Delete Sheet Columns", "home.deleteColumns", json!({})),
            ],
            Cluster::Editing => vec![
                ("AutoSum", "formulas.autoSum", json!({})),
                ("Find…", "dialog:find", json!({})),
                ("Replace…", "dialog:find", json!({"replace": true})),
            ],
        }
    }
}

/// Room kept at the row's end for the More tools button.
const MORE_W: f32 = 34.0;

/// The tool row: as many clusters as fit, then a More tools menu with the rest.
fn tools(app: &mut SheetApp, ui: &mut Ui) {
    let widths_id = egui::Id::new("nexus_tool_widths");
    let mut widths: Vec<f32> = ui.data(|d| d.get_temp::<Vec<f32>>(widths_id)).unwrap_or_default();
    widths.resize(Cluster::ALL.len(), 0.0);
    let width = |i: usize, c: Cluster| widths.get(i).copied().filter(|w| *w > 0.0).unwrap_or(c.estimate());
    let avail = ui.available_width();
    let total: f32 = Cluster::ALL.iter().enumerate().map(|(i, c)| width(i, *c)).sum();
    let mut shown = Cluster::ALL.len();
    if total > avail {
        let mut used = MORE_W;
        shown = 0;
        for (i, c) in Cluster::ALL.iter().enumerate() {
            used += width(i, *c);
            if used > avail {
                break;
            }
            shown = i + 1;
        }
    }
    let mut measured = widths.clone();
    for (i, c) in Cluster::ALL.iter().enumerate().take(shown) {
        let x0 = ui.cursor().min.x;
        if i > 0 {
            divider(ui);
        }
        draw(app, ui, *c);
        if let Some(m) = measured.get_mut(i) {
            *m = ui.cursor().min.x - x0;
        }
    }
    if shown < Cluster::ALL.len() {
        divider(ui);
        let more = icon_button(ui, Icon::More, Tokens::get(ui.ctx()).text, "More tools", vec2(28.0, 26.0));
        let more = more.on_hover_text(app.ui.language.tr("More tools"));
        more.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "More tools"));
        egui::Popup::menu(&more).show(|ui| {
            for (n, c) in Cluster::ALL.iter().enumerate().skip(shown) {
                if n > shown {
                    ui.separator();
                }
                let items = c.items(app);
                menu_items(app, ui, &items);
            }
        });
    }
    if measured != widths {
        ui.data_mut(|d| d.insert_temp(widths_id, measured));
    }
}

/// A thin divider between tool clusters.
fn divider(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add_space(4.0);
    let (r, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
    ui.painter().line_segment([r.center_top(), r.center_bottom()], Stroke::new(1.0, t.separator));
    ui.add_space(4.0);
}

/// Gives a tool its name for screen readers and tests (the widgets only set a tooltip).
fn named(resp: egui::Response, name: &str, selected: Option<bool>) -> egui::Response {
    let enabled = resp.enabled();
    resp.widget_info(|| match selected {
        Some(on) => egui::WidgetInfo::selected(egui::WidgetType::Checkbox, enabled, on, name),
        None => egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, name),
    });
    resp
}

fn draw(app: &mut SheetApp, ui: &mut Ui, c: Cluster) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language;
    let st = active_style(app);
    let size = vec2(26.0, 26.0);
    match c {
        Cluster::Clipboard => {
            let (p, pa) = split_button(ui, Icon::Paste, None, "Paste");
            if p {
                app.paste_from_clipboard();
            }
            let pa = named(pa, "Paste options", None);
            egui::Popup::menu(&pa).show(|ui| {
                menu_items(
                    app,
                    ui,
                    &[
                        ("Paste", "edit.paste", json!({})),
                        ("Paste Values", "edit.pasteSpecial", json!({"what": "values"})),
                        ("Paste Formulas", "edit.pasteSpecial", json!({"what": "formulas"})),
                        ("Paste Formatting", "edit.pasteSpecial", json!({"what": "formats"})),
                        ("Transpose", "edit.pasteSpecial", json!({"what": "all", "transpose": true})),
                        ("-", "", json!(null)),
                        ("Paste Special…", "dialog:pasteSpecial", json!({})),
                    ],
                );
            });
            if named(icon_button(ui, Icon::Cut, t.text, "Cut (⌘X)", size), "Cut", None).clicked() {
                app.copy_to_clipboard(ui.ctx(), "edit.cut");
            }
            if named(icon_button(ui, Icon::Copy, t.text, "Copy (⌘C)", size), "Copy", None).clicked() {
                app.copy_to_clipboard(ui.ctx(), "edit.copy");
            }
            let fp_on = app.session.format_painter.is_some();
            let fp = named(toggle_button(ui, Icon::Brush, fp_on, "Format Painter (double-click to keep it on)"), "Format Painter", Some(fp_on));
            if fp.double_clicked() {
                act(app, "edit.formatPainter", json!({"sticky": true}));
            } else if fp.clicked() {
                if fp_on {
                    app.session.format_painter = None;
                } else {
                    act(app, "edit.formatPainter", json!({}));
                }
            }
        }
        Cluster::Font => {
            font_combo(app, ui, &st);
            ui.add_space(2.0);
            size_combo(app, ui, &st);
        }
        Cluster::Format => {
            for (icon, on, id, name, tip) in [
                (Icon::Bold, st.font.bold, "home.bold", "Bold", "Bold (⌘B)"),
                (Icon::Italic, st.font.italic, "home.italic", "Italic", "Italic (⌘I)"),
                (Icon::Underline, st.font.underline != gridcraft_engine::model::Underline::None, "home.underline", "Underline", "Underline (⌘U)"),
            ] {
                if named(toggle_button(ui, icon, on, tip), name, Some(on)).clicked() {
                    act(app, id, json!({}));
                }
            }
            ui.add_space(2.0);
            let (b, ba) = split_button(ui, Icon::Borders, None, "Borders");
            if b {
                act(app, "home.borders", json!({"preset": app.grid.last_border.clone()}));
            }
            let ba = named(ba, "Border presets", None);
            egui::Popup::menu(&ba).show(|ui| {
                for (label, preset) in [
                    ("Bottom Border", "bottom"),
                    ("Top Border", "top"),
                    ("Left Border", "left"),
                    ("Right Border", "right"),
                    ("No Border", "none"),
                    ("All Borders", "all"),
                    ("Outside Borders", "outside"),
                    ("Thick Outside Borders", "thickOutside"),
                    ("Bottom Double Border", "doubleBottom"),
                    ("Inside Borders", "inside"),
                ] {
                    if crate::border_preview::preset_button(ui, label, preset).clicked() {
                        app.grid.last_border = preset.to_string();
                        act(app, "home.borders", json!({"preset": preset}));
                    }
                }
                ui.separator();
                if ui.button(lang.tr("More Borders…")).clicked() {
                    app.open_dialog("formatCells", json!({"tab": "Border"}));
                }
            });
            let fill = app.grid.last_fill.clone();
            let fill_c = gridcraft_engine::model::Color::from_hex(&fill).and_then(|c| c.resolve(&Default::default())).map(theme::color32);
            let (f, fa) = split_button(ui, Icon::Fill, fill_c, "Fill Color");
            if f {
                act(app, "home.fillColor", json!({"color": fill}));
            }
            let fa = named(fa, "Fill Color options", None);
            egui::Popup::menu(&fa).show(|ui| {
                if let Some(c) = color_palette(ui, &theme_colors(app), "No Fill") {
                    if c != "none" {
                        app.grid.last_fill = c.clone();
                    }
                    act(app, "home.fillColor", json!({"color": c}));
                    ui.close();
                }
            });
            let fc = app.grid.last_font_color.clone();
            let fc_c = gridcraft_engine::model::Color::from_hex(&fc).and_then(|c| c.resolve(&Default::default())).map(theme::color32);
            let (c, ca) = split_button(ui, Icon::FontColor, fc_c, "Font Color");
            if c {
                act(app, "home.fontColor", json!({"color": fc}));
            }
            let ca = named(ca, "Font Color options", None);
            egui::Popup::menu(&ca).show(|ui| {
                if let Some(c) = color_palette(ui, &theme_colors(app), "Automatic") {
                    if c != "none" {
                        app.grid.last_font_color = c.clone();
                    }
                    act(app, "home.fontColor", json!({"color": if c == "none" { "auto".to_string() } else { c }}));
                    ui.close();
                }
            });
        }
        Cluster::Align => {
            for (icon, h, id, name) in [
                (Icon::AlignLeft, HAlign::Left, "home.alignLeft", "Align Left"),
                (Icon::AlignCenter, HAlign::Center, "home.alignCenter", "Center"),
                (Icon::AlignRight, HAlign::Right, "home.alignRight", "Align Right"),
            ] {
                let on = st.align.h == h;
                if named(toggle_button(ui, icon, on, name), name, Some(on)).clicked() {
                    act(app, id, json!({}));
                }
            }
            if named(toggle_button(ui, Icon::Wrap, st.align.wrap, "Wrap Text"), "Wrap Text", Some(st.align.wrap)).clicked() {
                act(app, "home.wrapText", json!({}));
            }
            ui.add_space(2.0);
            let (m, ma) = split_button(ui, Icon::Merge, None, "Merge & Center");
            if m {
                act(app, "home.mergeCenter", json!({}));
            }
            let ma = named(ma, "Merge options", None);
            egui::Popup::menu(&ma).show(|ui| {
                menu_items(
                    app,
                    ui,
                    &[
                        ("Merge & Center", "home.mergeCenter", json!({})),
                        ("Merge Across", "home.mergeAcross", json!({})),
                        ("Merge Cells", "home.mergeCells", json!({})),
                        ("Unmerge Cells", "home.unmergeCells", json!({})),
                    ],
                );
            });
        }
        Cluster::Number => {
            let (c, ca) = split_button(ui, Icon::Currency, None, "Accounting Number Format");
            if c {
                act(app, "home.accounting", json!({}));
            }
            let ca = named(ca, "Number formats", None);
            egui::Popup::menu(&ca).show(|ui| {
                let names = [
                    "General",
                    "Number",
                    "Currency",
                    "Accounting",
                    "Short Date",
                    "Long Date",
                    "Time",
                    "Percentage",
                    "Fraction",
                    "Scientific",
                    "Text",
                ];
                for n in names {
                    if ui.add(egui::Button::new(lang.tr(n)).frame(false).min_size(vec2(200.0, 22.0))).clicked() {
                        act(app, "home.numberFormat", json!({"format": n}));
                        ui.close();
                    }
                }
                ui.separator();
                if ui.add(egui::Button::new(lang.tr("More Number Formats…")).frame(false).min_size(vec2(200.0, 22.0))).clicked() {
                    app.open_dialog("formatCells", json!({"tab": "Number"}));
                    ui.close();
                }
            });
            for (icon, id, name) in [
                (Icon::Percent, "home.percent", "Percent Style"),
                (Icon::Comma, "home.comma", "Comma Style"),
                (Icon::DecInc, "home.increaseDecimal", "Increase Decimal"),
                (Icon::DecDec, "home.decreaseDecimal", "Decrease Decimal"),
            ] {
                if named(icon_button(ui, icon, t.text, name, size), name, None).clicked() {
                    act(app, id, json!({}));
                }
            }
        }
        Cluster::SortFilter => {
            for (icon, id, name) in [(Icon::SortAsc, "data.sortAscending", "Sort A to Z"), (Icon::SortDesc, "data.sortDescending", "Sort Z to A")] {
                if named(icon_button(ui, icon, t.text, name, size), name, None).clicked() {
                    act(app, id, json!({}));
                }
            }
            let filtering = app.session.active().and_then(|d| d.wb.active().map(|sh| sh.autofilter.is_some())).unwrap_or(false);
            if named(toggle_button(ui, Icon::Filter, filtering, "Filter (⌘⇧L)"), "Filter", Some(filtering)).clicked() {
                act(app, "data.filter", json!({}));
            }
        }
        Cluster::Cells => {
            let ins = named(small_button(ui, Icon::Insert, "", "Insert cells, rows or columns", true), "Insert", None);
            egui::Popup::menu(&ins).show(|ui| {
                menu_items(
                    app,
                    ui,
                    &[
                        ("Insert Cells…", "dialog:insertCells", json!({})),
                        ("Insert Sheet Rows", "home.insertRows", json!({})),
                        ("Insert Sheet Columns", "home.insertColumns", json!({})),
                        ("Insert Sheet", "home.insertSheet", json!({})),
                    ],
                );
            });
            let del = named(small_button(ui, Icon::Delete, "", "Delete cells, rows or columns", true), "Delete", None);
            egui::Popup::menu(&del).show(|ui| {
                menu_items(
                    app,
                    ui,
                    &[
                        ("Delete Cells…", "dialog:deleteCells", json!({})),
                        ("Delete Sheet Rows", "home.deleteRows", json!({})),
                        ("Delete Sheet Columns", "home.deleteColumns", json!({})),
                        ("Delete Sheet", "home.deleteSheet", json!({})),
                    ],
                );
            });
        }
        Cluster::Editing => {
            let (s, sa) = split_button(ui, Icon::Sum, None, "AutoSum (⌘⇧T)");
            if s {
                act(app, "formulas.autoSum", json!({}));
            }
            let sa = named(sa, "AutoSum functions", None);
            egui::Popup::menu(&sa).show(|ui| {
                for f in ["SUM", "AVERAGE", "COUNT", "MAX", "MIN"] {
                    if ui.button(f).clicked() {
                        act(app, "formulas.autoSum", json!({"function": f}));
                    }
                }
                ui.separator();
                if ui.button(lang.tr("More Functions…")).clicked() {
                    app.open_dialog("insertFunction", json!({}));
                }
            });
            if named(icon_button(ui, Icon::Find, t.text, "Find & Replace (⌘F)", size), "Find", None).clicked() {
                app.open_dialog("find", json!({}));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(m: &Menu) -> Vec<&'static str> {
        m.groups
            .iter()
            .flat_map(|(_, es)| {
                es.iter().flat_map(|e| match e {
                    Entry::Command(id) => vec![*id],
                    Entry::Sub(_, ids) => ids.clone(),
                })
            })
            .collect()
    }

    /// Every command of a shown tab is in that tab's menu, under its ribbon group, so leaving the
    /// ribbon loses no command.
    #[test]
    fn menus_hold_every_command_of_their_tab() {
        let specs = gridcraft_engine::command_specs();
        let tabs = ["File", "Home", "Insert", "Data", "View", "No Such Tab"];
        let ms = menus(specs, &tabs);
        assert_eq!(ms.iter().map(|m| m.tab).collect::<Vec<_>>(), ["Home", "Insert", "Data", "View"], "File and empty tabs have no menu");
        for m in &ms {
            let listed = ids(m);
            let expected: Vec<&str> = specs.iter().filter(|c| c.menu.first().is_some_and(|f| menu_tab(f) == m.tab)).map(|c| c.id).collect();
            assert_eq!(listed.len(), expected.len(), "{}", m.tab);
            for id in expected {
                assert!(listed.contains(&id), "{id} in {}", m.tab);
            }
        }
        let home = &ms[0];
        assert!(home.groups.iter().any(|(g, es)| *g == "Font" && es.contains(&Entry::Command("home.bold"))), "{home:?}");
        // Home › Cells › Insert is a submenu of the Cells group; Edit's commands join Home.
        assert!(
            home.groups
                .iter()
                .any(|(g, es)| *g == "Cells" && es.iter().any(|e| matches!(e, Entry::Sub("Insert", ids) if ids.contains(&"home.insertRows"))))
        );
        assert!(ids(home).contains(&"edit.undo"));
    }

    /// With every tab the bar can show (the ribbon's, all contextual ones and PivotTable), every
    /// registry command placed anywhere but File is in some menu.
    #[test]
    fn every_placed_command_is_in_a_menu() {
        let specs = gridcraft_engine::command_specs();
        let mut tabs: Vec<&str> = crate::ribbon::TABS.to_vec();
        tabs.extend(["Table Design", "Chart Design", crate::paper_view::TAB, PIVOT]);
        let all: Vec<&str> = menus(specs, &tabs).iter().flat_map(ids).collect();
        for c in specs.iter().filter(|c| c.menu.first().is_some_and(|f| *f != "File")) {
            assert!(all.contains(&c.id), "{} ({:?}) is in no menu", c.id, c.menu);
        }
    }

    /// The menus are built from any registry slice, so an empty one gives no menu (no panic).
    #[test]
    fn no_commands_no_menus() {
        assert!(menus(&[], &["Home", "Insert"]).is_empty());
        assert!(menus(gridcraft_engine::command_specs(), &[]).is_empty());
    }

    fn harness(width: f32, height: f32) -> egui_kittest::Harness<'static, SheetApp> {
        let mut session = gridcraft_engine::Session::new();
        session.new_workbook();
        let mut app = SheetApp::new(session, crate::Services::default());
        app.toolbar = true;
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(width, height)).build_ui_state(
            |ui, app: &mut SheetApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app,
        );
        h.run_steps(8);
        h
    }

    fn bold(h: &mut egui_kittest::Harness<'static, SheetApp>) -> bool {
        active_style(h.state()).font.bold
    }

    /// The menu bar opens a tab's menu with its groups, a group lists its commands and a click
    /// runs the command; the tool row's Bold toggles bold. No ribbon tab row is drawn.
    #[test]
    fn menus_and_tools_work_in_the_window() {
        use egui_kittest::kittest::Queryable;
        let mut h = harness(1440.0, 900.0);
        assert!(!bold(&mut h));
        h.get_by_label("Bold").click();
        h.run_steps(6);
        assert!(bold(&mut h), "the tool row's Bold ran home.bold");
        // Home › Font › Bold, through the menus.
        h.get_by_label("Home").click();
        h.run_steps(6);
        h.get_by_label("Font ⏵").hover();
        h.run_steps(6);
        // Menu items carry their shortcut: "Bold" plus "Cmd+B".
        h.get_by_label_contains("Cmd+I");
        h.get_by_label_contains("Cmd+B").click();
        h.run_steps(6);
        assert!(!bold(&mut h), "the menu's Bold ran home.bold again");
        // Data › Sort & Filter lists Sort A to Z.
        h.get_by_label("Data").click();
        h.run_steps(6);
        h.get_by_label("Sort & Filter ⏵").hover();
        h.run_steps(6);
        assert_eq!(h.query_all_by_label("Sort A to Z").count(), 2, "the tool and the menu item");
    }

    /// A narrow window keeps the row on one line: the tools that don't fit move into More tools,
    /// and the ones there still run.
    #[test]
    fn a_narrow_window_moves_tools_into_more() {
        use egui_kittest::kittest::Queryable;
        let h = harness(1440.0, 900.0);
        assert!(h.query_by_label("More tools").is_none(), "everything fits at 1440");
        let mut h = harness(700.0, 700.0);
        assert!(h.query_by_label("Find").is_none(), "Find gave way");
        h.get_by_label("More tools").click();
        h.run_steps(6);
        h.get_by_label("AutoSum");
        h.get_by_label("Wrap Text").click();
        h.run_steps(6);
        assert!(active_style(h.state()).align.wrap, "Wrap Text ran from More tools");
    }

    /// `ui.accent` sets, reads and resets the accent; junk is refused; the palette follows.
    #[test]
    fn the_accent_command_sets_reads_and_resets() {
        let mut h = harness(1280.0, 800.0);
        let coral = egui::Color32::from_rgb(0xF4, 0x3F, 0x5E);
        assert_eq!(h.state_mut().run("ui.accent", json!({})), Ok(json!({"accent": null})));
        assert_eq!(h.state_mut().run("ui.accent", json!({"value": "#f43f5e"})), Ok(json!({"accent": "#F43F5E"})));
        h.run_steps(2);
        assert_eq!(theme::accent(&h.ctx), Some(coral));
        assert_eq!(Tokens::get(&h.ctx), theme::with_accent(Tokens::light(), coral));
        assert_eq!(h.state_mut().run("ui.accent", json!({})), Ok(json!({"accent": "#F43F5E"})), "no value reads it");
        for junk in [json!("red"), json!("#12345"), json!(42), json!([1, 2])] {
            assert!(h.state_mut().run("ui.accent", json!({"value": junk})).is_err(), "{junk}");
        }
        assert_eq!(h.state().ui.accent.as_deref(), Some("#F43F5E"), "junk leaves it");
        assert_eq!(h.state_mut().run("ui.accent", json!({"value": ""})), Ok(json!({"accent": null})));
        h.run_steps(2);
        assert_eq!(theme::accent(&h.ctx), None);
        h.state_mut().run("ui.accent", json!({"value": "10B981"})).unwrap();
        assert_eq!(h.state_mut().run("ui.accent", json!({"value": null})), Ok(json!({"accent": null})));
    }

    /// The accent is saved with the UI preferences; a saved value that isn't a colour reads as
    /// none, and no accent writes no key (an upstream `ui.json` stays the same).
    #[test]
    fn the_accent_round_trips_through_the_preferences() {
        let mut ui = crate::UiState { accent: Some("#F43F5E".into()), ..Default::default() };
        let json = serde_json::to_string(&ui).unwrap();
        let back: crate::UiState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.accent.as_deref(), Some("#F43F5E"));
        ui.accent = None;
        assert!(!serde_json::to_string(&ui).unwrap().contains("accent"));
        for (saved, read) in [(r##"{"accent":"#abcdef"}"##, Some("#ABCDEF")), (r#"{"accent":"pink"}"#, None), (r#"{"accent":7}"#, None), ("{}", None)]
        {
            let s: crate::UiState = serde_json::from_str(saved).unwrap();
            assert_eq!(s.accent.as_deref(), read, "{saved}");
        }
    }
}
