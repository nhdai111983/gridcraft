//! NexusOffices: the product names the interface shows.
//!
//! Upstream's English labels (the catalog keys) and some translations say "GridCraft" and
//! "ArtCraft". Rather than rewrite keys in eleven catalogs, [`super::Language::tr`] passes the
//! label it looked up through [`branded`] once, so every language, English included, shows
//! Nexus Sheets / NexusOffices. Keys, command ids, file keys and `GRIDCRAFT_*` variables are
//! untouched: only `GridCraft` and `ArtCraft` spelled exactly so are replaced, and only in what
//! `tr` returns (interface labels; `tr` is never given workbook text, see its doc comment).
//!
//! Text that must keep the upstream names (the attribution in About) is not looked up through
//! `tr` (see `crate::nexus`).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Upstream name → the name NexusOffices shows, in the order they are replaced.
const NAMES: [(&str, &str); 2] = [("GridCraft", "Nexus Sheets"), ("ArtCraft", "NexusOffices")];

/// At most this many distinct branded strings are kept ([`branded`]). Interface labels that name
/// the product number a handful; the cap only bounds memory if something unexpected were ever
/// passed through the lookup, which then simply comes back unbranded.
const MAX_INTERNED: usize = 1024;

/// `s` with the upstream product names replaced (`GridCraft` → `Nexus Sheets`, `ArtCraft` →
/// `NexusOffices`). Strings without them come back borrowed, without allocating.
pub fn brand(s: &str) -> Cow<'_, str> {
    if !NAMES.iter().any(|(from, _)| s.contains(from)) {
        return Cow::Borrowed(s);
    }
    let mut out = s.to_string();
    for (from, to) in NAMES {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    Cow::Owned(out)
}

/// [`brand`] for [`super::Language::tr`], which hands out `&str`: a label that needs branding is
/// branded once and kept for the life of the process (a bounded set: the interface labels).
pub(super) fn branded(s: &str) -> &str {
    if !NAMES.iter().any(|(from, _)| s.contains(from)) {
        return s;
    }
    static CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    // A poisoned lock still holds a usable map (inserts are single statements).
    let mut map = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(&hit) = map.get(s) {
        return hit;
    }
    if map.len() >= MAX_INTERNED {
        return s;
    }
    let leaked: &'static str = Box::leak(brand(s).into_owned().into_boxed_str());
    map.insert(s.to_string(), leaked);
    leaked
}

/// Text NexusOffices adds to the interface (the About page), written in English and Vietnamese:
/// Vietnamese when the interface language is Vietnamese (once the fork adds it), English
/// otherwise. Deliberately not a catalog key, so neither branded nor asked of upstream's catalogs
/// (the complete-catalog check in `i18n.rs`).
pub fn nexus_text<'a>(language: super::Language, en: &'a str, vi: &'a str) -> &'a str {
    if language.code() == "vi" { vi } else { en }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn brand_substitutes_product_names() {
        assert_eq!(brand("About GridCraft"), "About Nexus Sheets");
        assert_eq!(brand("Join the ArtCraft Discord"), "Join the NexusOffices Discord");
        assert_eq!(brand("Giới thiệu về GridCraft"), "Giới thiệu về Nexus Sheets");
        assert!(matches!(brand("Giới thiệu"), Cow::Borrowed("Giới thiệu")));
        // Identifiers and variables keep upstream's spelling.
        assert!(matches!(brand("gridcraft:// GRIDCRAFT_NO_PREFS gridcraft-cli"), Cow::Borrowed(_)));
    }

    #[test]
    fn branded_lookups_are_stable_and_borrow_when_unbranded() {
        let a = branded("How agents drive GridCraft (MCP / control channel)");
        let b = branded("How agents drive GridCraft (MCP / control channel)");
        assert_eq!(a, "How agents drive Nexus Sheets (MCP / control channel)");
        assert!(std::ptr::eq(a, b), "the branded string is made once");
        let plain = String::from("Close");
        assert!(std::ptr::eq(branded(&plain), plain.as_str()));
    }

    /// `tr` brands in every language, English included, and still falls back to the label.
    #[test]
    fn interface_lookups_show_nexus_names_in_every_language() {
        assert_eq!(Language::En.tr("About\nGridCraft"), "About\nNexus Sheets");
        assert_eq!(Language::En.tr("About GridCraft"), "About Nexus Sheets");
        assert_eq!(Language::En.tr("GridCraft"), "Nexus Sheets");
        assert_eq!(Language::Ja.tr("About\nGridCraft"), "Nexus Sheets について");
        assert_eq!(Language::PtBr.tr("About GridCraft"), "Sobre o Nexus Sheets");
        for language in Language::ALL {
            for key in ["About\nGridCraft", "About GridCraft", "How agents drive GridCraft (MCP / control channel)", "GridCraft"] {
                let shown = language.tr(key);
                assert!(!shown.contains("GridCraft") && !shown.contains("ArtCraft"), "{language:?}: {shown}");
            }
        }
        assert_eq!(Language::En.tr("Sheet1!A1"), "Sheet1!A1");
        assert_eq!(nexus_text(Language::En, "Source", "Mã nguồn"), "Source");
    }
}
