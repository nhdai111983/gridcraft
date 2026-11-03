//! NexusOffice: interface labels translated where they are drawn.
//!
//! Upstream draws many dialog, menu and status-bar labels as English literals, so a translation in
//! the catalogue (`i18n/vi.rs`) never reached the screen (QC-SHT-03, QC-SHT-06). Every such label
//! goes through [`t`] with the frame's language ([`crate::i18n::current`]); `tests/nexus_i18n_dialogs.rs`
//! opens every dialog in Vietnamese and fails on an English label.

/// `text` (an English interface label, never workbook content) in the language of the frame
/// being drawn.
pub fn t<'a>(ui: &egui::Ui, text: &'a str) -> &'a str {
    crate::i18n::current(ui.ctx()).tr(text)
}

/// A message for a message box or toast: its translation when the catalogue has one, else the
/// text unchanged. Unlike [`crate::i18n::Language::tr`], untranslated text is not branded, as a
/// message can quote a file or sheet name.
pub fn message(lang: crate::i18n::Language, text: &str) -> &str {
    let tr = lang.tr(text);
    if tr == crate::i18n::brand(text) { text } else { tr }
}

/// [`t`] for a label with `{}` placeholders filled in order (`"Sum: {}"`), translated before
/// filling so the catalogue key stays the English template.
pub fn tf(ui: &egui::Ui, template: &str, args: &[&str]) -> String {
    fill(crate::i18n::current(ui.ctx()).tr(template), args)
}

/// Replaces each `{}` in `template` with the next of `args` (extra `{}` stay; extra args are
/// ignored).
pub fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + args.iter().map(|a| a.len()).sum::<usize>());
    let mut rest = template;
    let mut args = args.iter();
    while let Some(i) = rest.find("{}") {
        let Some(a) = args.next() else { break };
        out.push_str(rest.get(..i).unwrap_or_default());
        out.push_str(a);
        rest = rest.get(i + 2..).unwrap_or_default();
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn fill_replaces_placeholders_in_order() {
        assert_eq!(super::fill("Sum: {}", &["3"]), "Sum: 3");
        assert_eq!(super::fill("{} of {}", &["1", "2"]), "1 of 2");
        assert_eq!(super::fill("{} of {}", &["1"]), "1 of {}");
        assert_eq!(super::fill("none", &["x"]), "none");
        assert_eq!(super::fill("Tổng: {}", &["1.234,5"]), "Tổng: 1.234,5");
    }
}
