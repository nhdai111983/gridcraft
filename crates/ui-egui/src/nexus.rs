//! NexusOffices branding for the egui front end, kept in one file so upstream's files change as
//! little as possible (the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched).
//!
//! Interface labels that name upstream's product go through [`crate::i18n::brand`] in
//! [`crate::i18n::Language::tr`]; this module holds what the lookup can't express: the names and
//! ids the app and its packaging use, and the About page.

use crate::i18n::{Language, nexus_text};

/// The product name shown in the window title, menus, message boxes and About.
pub const APP_NAME: &str = "Nexus Sheets";
/// The suite the app belongs to.
pub const SUITE_NAME: &str = "NexusOffices";
/// The publisher.
pub const PUBLISHER: &str = "Sonnytech";
/// Desktop app id (Wayland app id, X11 WM_CLASS, macOS bundle id, Linux desktop file).
pub const APP_ID: &str = "vn.sonnytech.nexus.sheets";
/// The settings folder's name on macOS (`~/Library/Application Support/…`) and Windows
/// (`%APPDATA%\…`); its own, so Nexus Sheets never reads or overwrites GridCraft's settings.
pub const SETTINGS_DIR: &str = "Nexus Sheets";
/// The settings folder's name under `$XDG_CONFIG_HOME` (Linux, BSD).
pub const SETTINGS_DIR_XDG: &str = "nexus-sheets";
/// Where the source of this build lives.
pub const SOURCE_URL: &str = "https://github.com/nhdai111983/gridcraft";

/// The About page's attribution to upstream, which the MIT/Apache licences and ArtCraft's brand
/// licence ask for (text only, no logo, no endorsement implied). Never branded.
pub const ATTRIBUTION: (&str, &str) = (
    "NexusOffices is developed from the open-source apps of ArtCraft (MIT/Apache-2.0 licences). Nexus Sheets is based on GridCraft.",
    "NexusOffices được phát triển dựa trên các ứng dụng mã nguồn mở của ArtCraft (giấy phép MIT/Apache-2.0). Nexus Sheets dựa trên GridCraft.",
);
/// Upstream's copyright notice (`NOTICE`), kept as upstream wrote it. Never branded.
pub const UPSTREAM_COPYRIGHT: &str = "Copyright (c) 2026 ArtCraft Team and the GridCraft contributors. MIT OR Apache-2.0.";

/// What Help ▸ Check for Updates… says: Nexus Sheets is updated with the NexusOffices suite,
/// never by checking or downloading GridCraft's releases (see `crate::updates_ui::start`).
pub fn updates_text(language: Language) -> &'static str {
    nexus_text(
        language,
        "Nexus Sheets is updated with NexusOffices. To get a newer version, install the latest NexusOffices release.",
        "Nexus Sheets được cập nhật cùng bộ NexusOffices. Để có phiên bản mới hơn, hãy cài bản NexusOffices mới nhất.",
    )
}

/// The About page's first tab: name, version, what it is, where the source is, the attribution
/// and the copyright notices.
pub fn about_ui(ui: &mut egui::Ui, language: Language) {
    ui.heading(APP_NAME);
    ui.label(format!("{} {}", nexus_text(language, "Version", "Phiên bản"), env!("CARGO_PKG_VERSION")));
    ui.add_space(4.0);
    ui.label(nexus_text(
        language,
        "A free, open-source spreadsheet.\nPart of the NexusOffices office suite by Sonnytech.",
        "Bảng tính miễn phí, mã nguồn mở.\nThuộc bộ ứng dụng văn phòng NexusOffices của Sonnytech.",
    ));
    ui.add_space(6.0);
    ui.hyperlink_to(nexus_text(language, "Source code: github.com/nhdai111983/gridcraft", "Mã nguồn: github.com/nhdai111983/gridcraft"), SOURCE_URL);
    ui.add_space(6.0);
    ui.label(egui::RichText::new(nexus_text(language, ATTRIBUTION.0, ATTRIBUTION.1)).small().weak());
    ui.label(egui::RichText::new(UPSTREAM_COPYRIGHT).small().weak());
    ui.label(egui::RichText::new("Nexus Sheets © 2026 Sonnytech.").small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_come_from_the_suite_not_upstream() {
        for language in [Language::En, Language::Vi] {
            let text = updates_text(language);
            assert!(text.contains("NexusOffices") && !text.contains("Craft"), "{language:?}: {text}");
        }
        assert!(updates_text(Language::Vi).contains("cập nhật"));
    }

    #[test]
    fn names_follow_the_branding_table() {
        assert_eq!(APP_NAME, "Nexus Sheets");
        assert_eq!(SUITE_NAME, "NexusOffices");
        assert_eq!(PUBLISHER, "Sonnytech");
        assert_eq!(APP_ID, "vn.sonnytech.nexus.sheets");
        assert_ne!(SETTINGS_DIR, "GridCraft", "never GridCraft's settings folder");
        assert_ne!(SETTINGS_DIR_XDG, "gridcraft", "never GridCraft's settings folder");
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
