//! NexusOffice (Henry, 03/11; review 05/11): Middle-aligned cell text is centred on its letters,
//! the box from the face's ascent to its descent, as Excel does, not on the row egui lays out,
//! which adds the face's line gap below the letters and so drew Middle text high.
//!
//! The oracle is the face itself (skrifa's ascent and descent at the drawn size) and the baseline
//! of the text shape the grid painted, not the grid's own arithmetic. The cell font is egui's
//! bundled Ubuntu Light given a line gap of a quarter em (about Calibri's 0.22), so the case that
//! went wrong is tested whatever fonts the machine or `CRAFT_FONTS_DIR` provide.

#[path = "common/mod.rs"]
#[allow(dead_code)] // The shared harness; this file uses part of it.
mod common;

use egui::Shape;
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_ui_egui::grid::Geo;
use serde_json::json;
use skrifa::MetadataProvider as _;

const TEXT: &str = "Middle";

fn harness(align: &str) -> common::Harness {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "B2", "input": TEXT})).unwrap();
    s.execute("home.fontSize", json!({"range": "B2", "size": 48})).unwrap();
    s.execute("home.rowHeight", json!({"rows": "2:2", "height": 120})).unwrap();
    s.execute(align, json!({"range": "B2"})).unwrap();
    s.execute("selection.set", json!({"range": "D8"})).unwrap();
    let mut h = common::harness_with(common::builder().with_size(egui::vec2(1200.0, 800.0)), common::app(s), 6);
    // After the app's own first font setup (pinned set), which it doesn't redo.
    let mut defs = gridcraft_ui_egui::theme::pinned_font_definitions();
    defs.font_data.insert(GAP_FACE.into(), std::sync::Arc::new(egui::FontData::from_owned(face_with_line_gap())));
    defs.families.get_mut(&egui::FontFamily::Name(gridcraft_ui_egui::theme::CELL.into())).expect("the cell role").insert(0, GAP_FACE.into());
    h.ctx.set_fonts(defs);
    h.run_steps(4);
    h
}

const GAP_FACE: &str = "test-line-gap";

/// Ubuntu Light with its line gap (hhea and OS/2) set to a quarter of its em.
fn face_with_line_gap() -> Vec<u8> {
    let mut bytes = epaint_default_fonts::UBUNTU_LIGHT.to_vec();
    let u16_at = |b: &[u8], at: usize| u16::from_be_bytes([b[at], b[at + 1]]);
    let u32_at = |b: &[u8], at: usize| u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]);
    let table = |b: &[u8], tag: &[u8; 4]| {
        (0..usize::from(u16_at(b, 4)))
            .map(|i| 12 + 16 * i)
            .find(|&rec| &b[rec..rec + 4] == tag)
            .map(|rec| u32_at(b, rec + 8) as usize)
            .expect("table")
    };
    let units_per_em = u16_at(&bytes, table(&bytes, b"head") + 18);
    let gap = (units_per_em / 4).to_be_bytes();
    let hhea_line_gap = table(&bytes, b"hhea") + 8;
    let os2_typo_line_gap = table(&bytes, b"OS/2") + 72;
    bytes[hhea_line_gap..hhea_line_gap + 2].copy_from_slice(&gap);
    bytes[os2_typo_line_gap..os2_typo_line_gap + 2].copy_from_slice(&gap);
    bytes
}

fn cell_rect(h: &common::Harness, cell: &str) -> egui::Rect {
    let app = h.state();
    let sheet = app.session.active().unwrap().wb.active().unwrap();
    let geo = Geo::new(sheet, app.grid.rect.unwrap(), app.view().scroll);
    geo.cell_rect(sheet, CellRef::parse(cell).unwrap())
}

/// Where the letters of the painted `TEXT` are, top and bottom (the face's ascent and descent
/// about the baseline), and the face's line gap, all in points.
fn letters(h: &common::Harness) -> (f32, f32, f32) {
    let text = h
        .output()
        .shapes
        .iter()
        .find_map(|c| match &c.shape {
            Shape::Text(t) if t.galley.job.text == TEXT => Some(t.clone()),
            _ => None,
        })
        .expect("the cell's text is painted");
    let row = &text.galley.rows[0];
    let glyph = row.glyphs.first().expect("a glyph");
    let baseline = text.pos.y + row.pos.y + glyph.pos.y;
    let font = &text.galley.job.sections[0].format.font_id;
    let data = h
        .ctx
        .fonts(|f| {
            let defs = f.definitions();
            let name = defs.families.get(&font.family)?.first()?;
            defs.font_data.get(name).cloned()
        })
        .expect("the cell font's face");
    let face = skrifa::FontRef::from_index(data.bytes(), data.index).unwrap();
    let m = face.metrics(skrifa::instance::Size::new(font.size), skrifa::instance::LocationRef::default());
    (baseline - m.ascent, baseline - m.descent, m.leading)
}

#[test]
fn middle_centres_the_letters_not_the_row() {
    let h = harness("home.alignMiddle");
    let rect = cell_rect(&h, "B2");
    let (top, bottom, gap) = letters(&h);
    // Without a gap there is nothing to tell apart: a quarter em of 48 pt is 12 pt.
    assert!(gap > 10.0, "the cell font is the test face, with its line gap ({gap} pt)");
    let off = (top + bottom) / 2.0 - rect.center().y;
    // The grid's own insets (1 pt above, 0.5 pt below at 100%) move it by under a point;
    // centring the whole row instead moves it up by half the gap.
    assert!(off.abs() < 1.0, "letters {top}..{bottom} in a cell {}..{}: {off} pt off centre (gap {gap})", rect.top(), rect.bottom());
}

/// Top and Bottom are unchanged: the gap only matters for Middle.
#[test]
fn top_and_bottom_still_hug_their_edge() {
    let h = harness("home.alignTop");
    let (top, _, _) = letters(&h);
    let rect = cell_rect(&h, "B2");
    assert!(top >= rect.top() - 2.0 && top < rect.top() + 12.0, "Top: letters start at {top} in a cell from {}", rect.top());
    let h = harness("home.alignBottom");
    let (_, bottom, gap) = letters(&h);
    let rect = cell_rect(&h, "B2");
    // Bottom keeps the row's gap under the letters, as before.
    assert!(bottom <= rect.bottom() && bottom > rect.bottom() - gap - 4.0, "Bottom: letters end at {bottom} in a cell to {}", rect.bottom());
}
