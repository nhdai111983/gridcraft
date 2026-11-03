//! NexusOffice branding for the egui front end, kept in one file so upstream's files change as
//! little as possible (the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched).
//!
//! Interface labels that name upstream's product go through [`crate::i18n::brand`] in
//! [`crate::i18n::Language::tr`]; this module holds what the lookup can't express: the names and
//! ids the app and its packaging use, and the About page.

use crate::i18n::{Language, nexus_text};

/// The product name shown in the window title, menus, message boxes and About.
pub const APP_NAME: &str = "Nexus Sheet";
/// The suite the app belongs to.
pub const SUITE_NAME: &str = "NexusOffice";
/// The publisher.
pub const PUBLISHER: &str = "SonnyTech";
/// Desktop app id (Wayland app id, X11 WM_CLASS, macOS bundle id, Linux desktop file).
pub const APP_ID: &str = "vn.sonnytech.nexus.sheets";
/// The settings folder's name on macOS (`~/Library/Application Support/…`) and Windows
/// (`%APPDATA%\…`); its own, so Nexus Sheet never reads or overwrites GridCraft's settings.
pub const SETTINGS_DIR: &str = "Nexus Sheet";
/// The settings folder's name under `$XDG_CONFIG_HOME` (Linux, BSD).
pub const SETTINGS_DIR_XDG: &str = "nexus-sheet";
/// The settings folder's name before the 2026-10-23 rename (`Nexus Sheets`), read as a fallback
/// when the new folder does not exist yet, so testers keep their preferences.
pub const LEGACY_SETTINGS_DIR: &str = "Nexus Sheets";
/// The same, under `$XDG_CONFIG_HOME`.
pub const LEGACY_SETTINGS_DIR_XDG: &str = "nexus-sheets";
/// Where the source of this build lives.
pub const SOURCE_URL: &str = "https://github.com/sonnytech-vn/gridcraft";

/// The About page's one-line description and suite sentence (English, Vietnamese).
pub const ABOUT_LINE: (&str, &str) = (
    "A free, open-source spreadsheet.\nPart of the NexusOffice suite by SonnyTech.",
    "Bảng tính miễn phí, mã nguồn mở.\nThuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech.",
);

/// The About page's attribution to upstream, which the MIT/Apache licences and ArtCraft's brand
/// licence ask for (text only, no logo, no endorsement implied). Never branded.
pub const ATTRIBUTION: (&str, &str) = (
    "NexusOffice is developed from the open-source apps of ArtCraft (MIT/Apache-2.0 licences). Nexus Sheet is based on GridCraft.",
    "NexusOffice được phát triển dựa trên các ứng dụng mã nguồn mở của ArtCraft (giấy phép MIT/Apache-2.0). Nexus Sheet dựa trên GridCraft.",
);
/// Upstream's copyright notice (`NOTICE`), kept as upstream wrote it. Never branded.
pub const UPSTREAM_COPYRIGHT: &str = "Copyright (c) 2026 ArtCraft Team and the GridCraft contributors. MIT OR Apache-2.0.";

/// What Help ▸ Check for Updates… says: Nexus Sheet is updated with the NexusOffice suite,
/// never by checking or downloading GridCraft's releases (see `crate::updates_ui::start`).
pub fn updates_text(language: Language) -> &'static str {
    nexus_text(
        language,
        "Nexus Sheet is updated with NexusOffice. To get a newer version, install the latest NexusOffice release.",
        "Nexus Sheet được cập nhật cùng bộ NexusOffice. Để có phiên bản mới hơn, hãy cài bản NexusOffice mới nhất.",
    )
}

/// The About page's first tab: name, version, what it is, where the source is, the attribution
/// and the copyright notices.
pub fn about_ui(ui: &mut egui::Ui, language: Language) {
    ui.heading(APP_NAME);
    ui.label(format!("{} {}", nexus_text(language, "Version", "Phiên bản"), env!("CARGO_PKG_VERSION")));
    ui.add_space(4.0);
    ui.label(nexus_text(language, ABOUT_LINE.0, ABOUT_LINE.1));
    ui.add_space(6.0);
    ui.hyperlink_to(
        nexus_text(language, "Source code: github.com/sonnytech-vn/gridcraft", "Mã nguồn: github.com/sonnytech-vn/gridcraft"),
        SOURCE_URL,
    );
    ui.add_space(6.0);
    ui.label(egui::RichText::new(nexus_text(language, ATTRIBUTION.0, ATTRIBUTION.1)).small().weak());
    ui.label(egui::RichText::new(UPSTREAM_COPYRIGHT).small().weak());
    ui.label(egui::RichText::new("Nexus Sheet © 2026 SonnyTech.").small().weak());
}

/// The line gap (`hhea` line gap / OS/2 typo line gap, as skrifa reads it) of `font_id`'s first
/// face, in points at its size: the room egui adds *below* the text in a row
/// (`row_height = ascent - descent + line_gap`, baseline at the ascent).
///
/// Henry, 03/11: Middle-aligned cell text sat high (Calibri's line gap is 0.22 em), because the
/// whole row was centred, not the letters. Excel centres the ascent-to-descent box; leaving the
/// gap out of the height does the same. 0 when the family or its face isn't known.
pub(crate) fn line_gap(ctx: &egui::Context, font_id: &egui::FontId) -> f32 {
    use skrifa::MetadataProvider as _;
    use std::collections::HashMap;
    use std::sync::{Mutex, PoisonError};
    /// The line gap per face name, as a fraction of the em.
    static GAP_EM: Mutex<Option<HashMap<String, f32>>> = Mutex::new(None);
    let face = ctx.fonts(|f| {
        let defs = f.definitions();
        let name = defs.families.get(&font_id.family)?.first()?.clone();
        let data = defs.font_data.get(&name)?.clone();
        Some((name, data))
    });
    let Some((name, data)) = face else { return 0.0 };
    let mut cache = GAP_EM.lock().unwrap_or_else(PoisonError::into_inner);
    let gaps = cache.get_or_insert_with(HashMap::new);
    let em = *gaps.entry(name).or_insert_with(|| {
        skrifa::FontRef::from_index(data.bytes(), data.index)
            .ok()
            .map(|font| {
                let m = font.metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
                let upem = f32::from(m.units_per_em.max(1));
                (m.leading / upem).clamp(0.0, 1.0)
            })
            .unwrap_or(0.0)
    });
    let gap = em * font_id.size;
    if gap.is_finite() { gap.max(0.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Henry, 03/11: the gap egui leaves under a row is measured from the face itself (Hack has
    /// none, Ubuntu has one), so Middle can centre the letters and not the row.
    #[test]
    fn line_gap_comes_from_the_face() {
        let ctx = egui::Context::default();
        let mut defs = egui::FontDefinitions::empty();
        for (name, bytes) in [("hack", epaint_default_fonts::HACK_REGULAR), ("ubuntu", epaint_default_fonts::UBUNTU_LIGHT)] {
            defs.font_data.insert(name.into(), std::sync::Arc::new(egui::FontData::from_static(bytes)));
            defs.families.insert(egui::FontFamily::Name(name.into()), vec![name.into()]);
        }
        ctx.set_fonts(defs);
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        let at = |family: &str| line_gap(&ctx, &egui::FontId::new(20.0, egui::FontFamily::Name(family.into())));
        for (family, bytes) in [("hack", epaint_default_fonts::HACK_REGULAR), ("ubuntu", epaint_default_fonts::UBUNTU_LIGHT)] {
            use skrifa::MetadataProvider as _;
            let m = skrifa::FontRef::new(bytes).unwrap().metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
            let want = m.leading / f32::from(m.units_per_em) * 20.0;
            assert!((at(family) - want).abs() < 1e-3, "{family}: {} vs {want}", at(family));
        }
        assert!(at("ubuntu") > 0.0, "Ubuntu has a line gap (Calibri's is 0.22 em)");
        assert_eq!(at("no such family"), 0.0);
    }

    #[test]
    fn updates_come_from_the_suite_not_upstream() {
        for language in [Language::En, Language::Vi] {
            let text = updates_text(language);
            assert!(text.contains("NexusOffice") && !text.contains("Craft"), "{language:?}: {text}");
        }
        assert!(updates_text(Language::Vi).contains("cập nhật"));
    }

    #[test]
    fn names_follow_the_branding_table() {
        assert_eq!(APP_NAME, "Nexus Sheet");
        assert_eq!(SUITE_NAME, "NexusOffice");
        assert_eq!(PUBLISHER, "SonnyTech");
        assert_eq!(APP_ID, "vn.sonnytech.nexus.sheets");
        assert_ne!(SETTINGS_DIR, "GridCraft", "never GridCraft's settings folder");
        assert_ne!(SETTINGS_DIR_XDG, "gridcraft", "never GridCraft's settings folder");
        assert_eq!(SOURCE_URL, "https://github.com/sonnytech-vn/gridcraft");
        assert!(ABOUT_LINE.0.ends_with("Part of the NexusOffice suite by SonnyTech."));
        assert!(ABOUT_LINE.1.ends_with("Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech."));
    }

    /// About attributes upstream in plain text (no logo, no link to ArtCraft) in both languages.
    #[test]
    fn about_attributes_upstream_in_text() {
        for (text, lang) in [(ATTRIBUTION.0, "en"), (ATTRIBUTION.1, "vi")] {
            assert!(text.contains("ArtCraft") && text.contains("GridCraft") && text.contains("MIT/Apache-2.0"), "{lang}: {text}");
            assert!(!text.contains("http"), "{lang}: text only");
        }
        assert!(UPSTREAM_COPYRIGHT.contains("ArtCraft Team"), "upstream's copyright kept");
        assert!(!SOURCE_URL.contains("artcraft"));
    }
}
