//! NexusOffice: Nexus Sheet's own colours (Emerald, the SonnyTech gradient) and the accent colour
//! the user picks in File › Options. Kept out of `theme.rs` so upstream's palette file changes as
//! little as possible; `theme` re-exports everything here.

use egui::Color32;

use super::{Tokens, install_visuals};

/// Nexus Sheet's app colour, Emerald (NexusOffice design system), and its ink shade for text and
/// filled controls (white on it reads at 5.5:1). Never Excel's green.
pub const APP_COLOR: Color32 = Color32::from_rgb(0x10, 0xB9, 0x81);
pub const APP_INK: Color32 = Color32::from_rgb(0x04, 0x78, 0x57);
/// The SonnyTech brand gradient (indigo → cyan → emerald), a thin line under the menu bar.
pub const BRAND_GRADIENT: [Color32; 3] =
    [Color32::from_rgb(0x63, 0x66, 0xF1), Color32::from_rgb(0x06, 0xB6, 0xD4), Color32::from_rgb(0x10, 0xB9, 0x81)];

/// Accent presets offered in File › Options (the NexusOffice palette); any other colour comes
/// from the picker.
pub const ACCENT_PRESETS: [(&str, Color32); 6] = [
    ("Emerald", APP_COLOR),
    ("Indigo", Color32::from_rgb(0x63, 0x66, 0xF1)),
    ("Violet", Color32::from_rgb(0x8B, 0x5C, 0xF6)),
    ("Sky", Color32::from_rgb(0x0E, 0xA5, 0xE9)),
    ("Amber", Color32::from_rgb(0xF5, 0x9E, 0x0B)),
    ("Coral", Color32::from_rgb(0xF4, 0x3F, 0x5E)),
];

/// Where the user's accent colour lives: the egui context's memory, so every widget reads it
/// through [`Tokens::get`] and each window (and each test harness) has its own.
fn accent_id() -> egui::Id {
    egui::Id::unique("nexus_user_accent")
}

/// The user's accent colour in this context, if any.
pub fn accent(ctx: &egui::Context) -> Option<Color32> {
    ctx.data(|d| d.get_temp::<Color32>(accent_id()))
}

/// Sets (or with `None` clears) the user's accent colour and re-installs the palettes. Returns
/// whether it changed.
pub fn set_accent(ctx: &egui::Context, c: Option<Color32>) -> bool {
    if accent(ctx) == c {
        return false;
    }
    ctx.data_mut(|d| {
        if let Some(c) = c {
            d.insert_temp(accent_id(), c);
        } else {
            d.remove::<Color32>(accent_id());
        }
    });
    install_visuals(ctx);
    ctx.request_repaint();
    true
}

/// `#RRGGBB` or `RRGGBB` → colour; anything else `None`.
pub fn parse_hex(s: &str) -> Option<Color32> {
    let h = s.trim();
    let h = h.strip_prefix('#').unwrap_or(h);
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| h.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok());
    Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// `#RRGGBB` for a colour.
pub fn to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// WCAG contrast ratio of two opaque colours (1 to 21): upstream's [`super::contrast`].
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    super::contrast(a, b)
}

/// `c` moved towards black (or white with `lighten`) in 2.5% steps until it reads against every
/// colour in `against` at `ratio`. Black and white end the walk, so it always stops.
fn shade_for(c: Color32, against: &[Color32], ratio: f32, lighten: bool) -> Color32 {
    let target = if lighten { 255.0 } else { 0.0 };
    let mix = |v: u8, t: f32| {
        let v = f32::from(v);
        (v + (target - v) * t).round().clamp(0.0, 255.0) as u8
    };
    for k in 0..=40u8 {
        let t = f32::from(k) * 0.025;
        let out = Color32::from_rgb(mix(c.r(), t), mix(c.g(), t), mix(c.b(), t));
        if against.iter().all(|b| contrast_ratio(out, *b) >= ratio) {
            return out;
        }
    }
    if lighten { Color32::WHITE } else { Color32::BLACK }
}

/// `c` mixed with white (`k` = how much of `c` stays).
fn tint(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (255.0 - (255.0 - f32::from(v)) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// `c` mixed with black (`k` = how much of `c` stays).
fn shade(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (f32::from(v) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// The palette with the user's accent colour `c` in place of the app colour. Every colour drawn
/// as text or under text is shaded until it reads at WCAG 4.5:1: the accent with its own text
/// (`on_accent`) and as text on the bars, the dark accent on the soft tint, the selected-header
/// text; the cell selection outline reads at 3:1 on the white cells.
pub fn with_accent(mut t: Tokens, c: Color32) -> Tokens {
    let lighten = t.dark;
    t.accent = shade_for(c, &[t.on_accent, t.window, t.ribbon, t.tab_bar, t.status_bar, t.menu_bg], 4.5, lighten);
    t.accent_soft = if t.dark { shade(c, 0.3) } else { tint(c, 0.16) };
    t.accent_dark = shade_for(c, &[t.accent_soft], 4.5, lighten);
    t.header_sel_text = shade_for(c, &[t.header_sel_bg], 4.5, lighten);
    t.header_all_bg = if t.dark { shade(c, 0.4) } else { tint(c, 0.35) };
    t.sel_border = shade_for(c, &[t.grid_bg], 3.0, false);
    t.sel_fill = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), if t.dark { 0x24 } else { 0x1C });
    t
}

#[cfg(test)]
mod tests {
    use super::{ACCENT_PRESETS, APP_COLOR, APP_INK, contrast_ratio, parse_hex, set_accent, to_hex, with_accent};
    use crate::theme::{Tokens, apply};
    use egui::Color32;

    /// Every colour drawn as text or under text keeps 4.5:1 with any accent the user picks, in
    /// both themes (and the selection outline 3:1 on the cells).
    #[test]
    fn user_accents_stay_readable() {
        let mut picks: Vec<Color32> = ACCENT_PRESETS.iter().map(|p| p.1).collect();
        picks.extend([Color32::WHITE, Color32::BLACK, Color32::YELLOW, Color32::from_rgb(0, 0, 0x40), Color32::from_rgb(0x80, 0x80, 0x80)]);
        for r in (0..=255u16).step_by(51) {
            for g in (0..=255u16).step_by(51) {
                for b in (0..=255u16).step_by(51) {
                    picks.push(Color32::from_rgb(r as u8, g as u8, b as u8));
                }
            }
        }
        for c in picks {
            for base in [Tokens::light(), Tokens::dark()] {
                let t = with_accent(base, c);
                for (what, fg, bg) in [
                    ("on_accent", t.on_accent, t.accent),
                    ("accent on window", t.accent, t.window),
                    ("accent on ribbon", t.accent, t.ribbon),
                    ("accent on tab bar", t.accent, t.tab_bar),
                    ("accent on menu", t.accent, t.menu_bg),
                    ("accent_dark on accent_soft", t.accent_dark, t.accent_soft),
                    ("header_sel_text", t.header_sel_text, t.header_sel_bg),
                ] {
                    let r = contrast_ratio(fg, bg);
                    assert!(r >= 4.5, "{c:?} dark={}: {what} {r:.2}", t.dark);
                }
                assert!(contrast_ratio(t.sel_border, t.grid_bg) >= 3.0, "{c:?}: selection outline");
            }
        }
    }

    /// The built-in Nexus palettes meet the same bar, and the app colour is Emerald, never
    /// Excel's green.
    #[test]
    fn nexus_palettes_are_readable_and_emerald() {
        for t in [Tokens::light(), Tokens::dark()] {
            assert!(contrast_ratio(t.on_accent, t.accent) >= 4.5, "dark={}", t.dark);
            assert!(contrast_ratio(t.accent, t.window) >= 4.5, "dark={}", t.dark);
            assert!(contrast_ratio(t.accent, t.ribbon) >= 4.5, "dark={}", t.dark);
            assert!(contrast_ratio(t.text, t.window) >= 4.5 && contrast_ratio(t.text_dim, t.ribbon) >= 4.5, "dark={}", t.dark);
            for c in [t.accent, t.accent_dark, t.sel_border, t.header_sel_text] {
                assert_ne!(c, Color32::from_rgb(0x21, 0x73, 0x46));
                assert_ne!(c, Color32::from_rgb(0x10, 0x7C, 0x41));
            }
        }
        assert_eq!(Tokens::light().accent, APP_INK);
        assert_eq!(to_hex(APP_COLOR), "#10B981");
    }

    #[test]
    fn hex_colours_parse_strictly() {
        assert_eq!(parse_hex("#10b981"), Some(Color32::from_rgb(0x10, 0xB9, 0x81)));
        assert_eq!(parse_hex(" F43F5E "), Some(Color32::from_rgb(0xF4, 0x3F, 0x5E)));
        for junk in ["", "#", "#12345", "#1234567", "zzzzzz", "#ÿÿÿ", "+12345", "#-12345", "##123456"] {
            assert_eq!(parse_hex(junk), None, "{junk:?}");
        }
        assert_eq!(to_hex(Color32::from_rgb(1, 2, 255)), "#0102FF");
    }

    /// The accent lives in the context: setting it changes the palette widgets read and the
    /// installed visuals; clearing it goes back to the app's own colour.
    #[test]
    fn the_accent_is_per_context() {
        let ctx = egui::Context::default();
        apply(&ctx, false);
        let other = egui::Context::default();
        apply(&other, false);
        let coral = Color32::from_rgb(0xF4, 0x3F, 0x5E);
        assert!(set_accent(&ctx, Some(coral)));
        assert!(!set_accent(&ctx, Some(coral)), "no change");
        let t = Tokens::get(&ctx);
        assert_eq!(t, with_accent(Tokens::light(), coral));
        assert_eq!(Tokens::get(&ctx), t, "the cached palette");
        assert_eq!(ctx.global_style().visuals.selection.bg_fill, t.accent_soft);
        assert_eq!(Tokens::get(&other), Tokens::light(), "another window keeps its own");
        assert!(set_accent(&ctx, None));
        assert_eq!(Tokens::get(&ctx), Tokens::light());
    }
}
