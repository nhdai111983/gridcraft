//! Regional formats (NexusOffice): how numbers, dates and times are shown and typed.
//!
//! A [`Locale`] only changes what people see and type. Files, formulas and the command API stay
//! en-US (OOXML stores numbers with `.` and formulas with `,`), so a workbook reads the same in
//! every locale. The interface picks the locale; nothing here is saved in a workbook.
//!
//! Vietnamese names follow the Unicode CLDR `vi` data (wide, abbreviated and narrow forms), the
//! separators the `vi-VN` regional format (decimal `,`, digit grouping `.`, list separator `;`).

use crate::date::{MONTHS, WEEKDAYS};

/// A regional format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Locale {
    /// English (United States): `1,234.5`, `m/d/yyyy`, `h:mm AM/PM`. The file and API format.
    #[default]
    EnUs,
    /// Vietnamese (Vietnam): `1.234,5`, `dd/mm/yyyy`, 24-hour `h:mm`, `1.234 ₫`.
    ViVn,
}

const VI_MONTHS: [&str; 12] =
    ["tháng 1", "tháng 2", "tháng 3", "tháng 4", "tháng 5", "tháng 6", "tháng 7", "tháng 8", "tháng 9", "tháng 10", "tháng 11", "tháng 12"];
const VI_MONTHS_ABBR: [&str; 12] = ["thg 1", "thg 2", "thg 3", "thg 4", "thg 5", "thg 6", "thg 7", "thg 8", "thg 9", "thg 10", "thg 11", "thg 12"];
const VI_MONTHS_NARROW: [&str; 12] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"];
/// Sunday first, as [`crate::date::DateTime::weekday`] counts.
const VI_WEEKDAYS: [&str; 7] = ["Chủ Nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"];
const VI_WEEKDAYS_ABBR: [&str; 7] = ["CN", "Th 2", "Th 3", "Th 4", "Th 5", "Th 6", "Th 7"];

/// The Vietnamese đồng's sign.
pub const DONG: char = '₫';

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::EnUs, Locale::ViVn];

    /// BCP 47 tag (`en-US`, `vi-VN`).
    pub const fn tag(self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::ViVn => "vi-VN",
        }
    }

    /// Reads `en`, `en-US`, `vi`, `vi_VN`… (case-insensitive). Other languages have no regional
    /// format of their own yet: `None`.
    pub fn from_tag(tag: &str) -> Option<Locale> {
        let lower = tag.trim().to_ascii_lowercase();
        match lower.split(['-', '_']).next()? {
            "en" => Some(Locale::EnUs),
            "vi" => Some(Locale::ViVn),
            _ => None,
        }
    }

    pub const fn decimal_separator(self) -> char {
        match self {
            Locale::EnUs => '.',
            Locale::ViVn => ',',
        }
    }

    pub const fn group_separator(self) -> char {
        match self {
            Locale::EnUs => ',',
            Locale::ViVn => '.',
        }
    }

    /// Separates fields in delimited text (CSV) and arguments in formulas.
    pub const fn list_separator(self) -> char {
        match self {
            Locale::EnUs => ',',
            Locale::ViVn => ';',
        }
    }

    /// The first day of a calendar week: 0 = Sunday, 1 = Monday.
    pub const fn first_weekday(self) -> u32 {
        match self {
            Locale::EnUs => 0,
            Locale::ViVn => 1,
        }
    }

    /// Month name for `mmmm`; `month` is 1..=12 (anything else is empty).
    pub fn month_name(self, month: u32) -> &'static str {
        let i = (month as usize).wrapping_sub(1);
        match self {
            Locale::EnUs => MONTHS.get(i).copied().unwrap_or(""),
            Locale::ViVn => VI_MONTHS.get(i).copied().unwrap_or(""),
        }
    }

    /// Month for `mmm` (`Oct`, `thg 10`).
    pub fn month_abbr(self, month: u32) -> &'static str {
        let i = (month as usize).wrapping_sub(1);
        match self {
            Locale::EnUs => MONTHS.get(i).copied().and_then(|m| m.get(..3)).unwrap_or(""),
            Locale::ViVn => VI_MONTHS_ABBR.get(i).copied().unwrap_or(""),
        }
    }

    /// Month for `mmmmm` (`O`, `10`: CLDR's narrow Vietnamese months are numbers).
    pub fn month_narrow(self, month: u32) -> &'static str {
        let i = (month as usize).wrapping_sub(1);
        match self {
            Locale::EnUs => MONTHS.get(i).copied().and_then(|m| m.get(..1)).unwrap_or(""),
            Locale::ViVn => VI_MONTHS_NARROW.get(i).copied().unwrap_or(""),
        }
    }

    /// Day name for `dddd`; `weekday` 0 = Sunday.
    pub fn day_name(self, weekday: u32) -> &'static str {
        let i = weekday as usize;
        match self {
            Locale::EnUs => WEEKDAYS.get(i).copied().unwrap_or(""),
            Locale::ViVn => VI_WEEKDAYS.get(i).copied().unwrap_or(""),
        }
    }

    /// Day for `ddd` (`Sat`, `Th 7`).
    pub fn day_abbr(self, weekday: u32) -> &'static str {
        let i = weekday as usize;
        match self {
            Locale::EnUs => WEEKDAYS.get(i).copied().and_then(|d| d.get(..3)).unwrap_or(""),
            Locale::ViVn => VI_WEEKDAYS_ABBR.get(i).copied().unwrap_or(""),
        }
    }

    /// What `AM/PM` shows (`SA`/`CH` in Vietnamese).
    pub const fn am_pm(self) -> (&'static str, &'static str) {
        match self {
            Locale::EnUs => ("AM", "PM"),
            Locale::ViVn => ("SA", "CH"),
        }
    }

    /// Respells en-US number text (`1,234.5`, `-1.5E+20`) in this locale (`1.234,5`). Only the
    /// two separators change; every other character is kept.
    pub fn localize_number(self, en: &str) -> String {
        self.swap_separators(en)
    }

    /// Respells number text typed in this locale (`1.234,5`) as en-US (`1,234.5`).
    pub fn delocalize_number(self, local: &str) -> String {
        self.swap_separators(local)
    }

    fn swap_separators(self, s: &str) -> String {
        match self {
            Locale::EnUs => s.to_string(),
            Locale::ViVn => s
                .chars()
                .map(|c| match c {
                    '.' => ',',
                    ',' => '.',
                    other => other,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags() {
        assert_eq!(Locale::from_tag("vi"), Some(Locale::ViVn));
        assert_eq!(Locale::from_tag(" VI_vn "), Some(Locale::ViVn));
        assert_eq!(Locale::from_tag("en-GB"), Some(Locale::EnUs));
        assert_eq!(Locale::from_tag("es"), None);
        assert_eq!(Locale::from_tag(""), None);
        for l in Locale::ALL {
            assert_eq!(Locale::from_tag(l.tag()), Some(l));
        }
    }

    #[test]
    fn names_and_separators() {
        assert_eq!(Locale::ViVn.day_name(1), "Thứ Hai");
        assert_eq!(Locale::ViVn.day_name(0), "Chủ Nhật");
        assert_eq!(Locale::ViVn.month_name(10), "tháng 10");
        assert_eq!(Locale::ViVn.month_abbr(1), "thg 1");
        assert_eq!(Locale::EnUs.month_abbr(10), "Oct");
        assert_eq!(Locale::EnUs.day_abbr(6), "Sat");
        assert_eq!(Locale::EnUs.month_narrow(10), "O");
        // Out of range never panics.
        assert_eq!(Locale::ViVn.month_name(0), "");
        assert_eq!(Locale::ViVn.month_name(13), "");
        assert_eq!(Locale::ViVn.day_abbr(7), "");
        assert_eq!(Locale::ViVn.localize_number("-1,234.5E+20"), "-1.234,5E+20");
        assert_eq!(Locale::EnUs.localize_number("1,234.5"), "1,234.5");
        assert_eq!(Locale::ViVn.first_weekday(), 1);
    }

    #[test]
    fn vietnamese_names_are_nfc() {
        // Precomposed letters only (NFC): no combining marks U+0300..U+036F.
        for w in 0..7 {
            assert!(!Locale::ViVn.day_name(w).chars().any(|c| ('\u{300}'..='\u{36f}').contains(&c)));
        }
        assert_eq!("Thứ Hai".chars().count(), 7);
    }
}
