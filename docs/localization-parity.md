# Localization parity

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-24 · **Change:** minor (NexusOffice: vi-VN regional format — numbers, đồng, dates, CSV, formula separators) · **Target:** Microsoft Excel (Microsoft 365), Excel for Mac 16.113.4

Per-language status against Excel, which ships its UI in dozens of languages (Windows language
packs cover 100+), with localized function names, argument separators, number and date formats,
right-to-left sheets and full complex-script support. Part of
[target-app-parity.md](target-app-parity.md). **Localization: ~10% ready, 100–165 h** (estimated).

## Where we are (measured from source on `main`, after the 2026-10-10 merges)

- **A first catalog exists, for the ribbon only.** `crates/ui-egui/src/i18n.rs` (merged 10-10 in
  #31, #104, #82; German added 10-11) adds an interface-language preference that defaults to the
  system language (closes #20) and a View-tab switcher, with English-keyed lookup tables:
  Japanese 268 labels, Simplified Chinese, Korean, Russian, Brazilian Portuguese, German and Czech 266
  each. The tables are used by
  the ribbon, sheet tabs and widgets (`ribbon.rs`, `tabs.rs`, `widgets.rs`), the Format Cells
  dialog, every dialog title and the shared OK/Cancel buttons; the other dialogs, menus,
  context menus, task panes, status messages, command labels and function descriptions are still
  English literals. Untranslated labels fall back to English.
- **About 2,200 user-visible strings** in all (heuristic source scan: ~1,700 UI and command
  strings plus ~500 function descriptions), so each shipped language covers **~12%** (measured
  count / estimated total).
- **en-US only formulas and formats.** Function names, `,` argument separator, `.` decimal
  separator, en-US number and date formats (`crates/numfmt` says "en-US only"). Users ask for
  localized function names and `;` separators (#45). German with localized numbers and formulas
  is in PR #163 (open).
- **Script support.** System-font fallback for CJK, Thai and Hebrew landed in #38 (closes #32;
  #142 is still open pending confirmation). There is no complex-text shaping: Arabic and
  Devanagari don't join or reorder, and there is no bidirectional layout. **Sheet Right-to-Left**
  (`view.rightToLeft`, Page Layout › Sheet Options, XLSX `rightToLeft`) mirrors the grid (#146):
  column A and the row headers on the right, the vertical scrollbar on the left; hit-testing,
  arrow keys, editor, popups and floating objects follow; text stays left-to-right inside its box
  (no bidi), and the sheet tabs, horizontal scrollbar and printing/PDF are not mirrored yet. IME input goes
  through egui/winit and is unverified for Chinese, Japanese and Korean.
- **PDF export** uses only the 14 standard PDF fonts: non-Latin text cannot export.

## Languages

UI strings translated are measured counts against the ~2,200 estimate. Estimates to `full`
include the rest of the catalog work below (10–20 h, counted once), localized function names,
separators, number/date formats, machine translation with agent review, and script work.

| Language | Code | UI strings translated | Dialogs / tooltips / help | Script support | Native review | Status | Estimate to full |
|---|---|---|---|---|---|---|---|
| English | en | ~2,200 / ~2,200 (100%, source) | yes | Latin: yes | yes | full | — |
| Simplified Chinese (Mandarin) | zh-Hans | 266 (~12%) | no | CJK system-font fallback (#38); IME unverified; PDF can't export | no | menus only | 8–13 h |
| Spanish | es | 0 (0%) | no | Latin: yes | no | none | 6–10 h |
| Hindi | hi | 0 (0%) | no | Devanagari: **no shaping** | no | none | 12–18 h |
| Arabic | ar | 0 (0%) | no | **no shaping, no RTL UI**; Sheet Right-to-Left mirrors the grid (#146) | no | none | 25–35 h |
| French | fr | 0 (0%) | no | Latin: yes | no | none | 6–10 h |
| Portuguese | pt-BR | 923 (~42%) | dialogs and tooltips (dialogs take effect with #669) | Latin: yes | no | menus, File view, dialogs | 2–4 h |
| Indonesian | id | 0 (0%) | no | Latin: yes | no | none | 5–8 h |
| Japanese | ja | 268 (~12%) | no | CJK system-font fallback; IME unverified; no furigana display | no | menus only | 8–12 h |
| German | de | 266 (~12%) | no | Latin: yes | no | menus only | 4–8 h |
| Korean | ko | 266 (~12%) | no | Hangul via the CJK fallback; IME unverified | no | menus only | 6–10 h |
| Vietnamese | vi | 960 (NexusOffice catalog `i18n/vi.rs`) | dialogs partly | Latin with stacked diacritics: unverified | no | menus, dialogs; **vi-VN regional format** (below) | 3–6 h |

Other shipped languages: Russian (ru) and German (de), 266 labels each (~12%), menus only.

**Total:** catalog work 10–20 h + languages 92–144 h ≈ **100–165 h**. Parallelizes almost fully
once the catalog covers dialogs and commands. **Needs a human:** native-speaker review of every
language (none so far) and of localized function names.

## Regional formats (NexusOffice: vi-VN)

A regional format (`gridcraft_core::Locale`: `EnUs`, `ViVn`) changes only what people see and
type. Files, stored formulas, number-format codes and the command API stay en-US.

- **Which one:** it follows the interface language (Vietnamese → vi-VN, every other language →
  en-US; Spanish keeps its formula syntax with en-US display). The app has no separate regional
  setting (Excel uses the system's): `Language::regional_format`, pushed every frame with
  `Session::set_locale`, which recalculates workbooks whose locale changes and keeps a saved
  workbook saved (`Workbook::locale` is `#[serde(skip)]`).
- **Display** (`numfmt::format_value_in`): decimal `,`, grouping `.` (`1.234.567,89`), General
  `1234,5`; CLDR `vi` month and day names (`tháng 10`/`thg 10`/`10`, `Thứ Hai`…`Chủ Nhật`/`Th 2`…`CN`),
  `SA`/`CH` for AM/PM, fractional seconds `ss,00`. The system formats follow the locale as in
  Excel: built-in 14 `m/d/yyyy` shows `dd/mm/yyyy`, 22 `dd/mm/yyyy h:mm`, `[$-F800]`/`[$-x-sysdate]`
  `dddd, d mmmm, yyyy`, `[$-F400]`/`[$-x-systime]` `h:mm:ss` (24 h). Explicit codes keep their shape.
  Calendar weeks start on Monday (`Locale::first_weekday`, no date picker uses it yet).
- **Currency:** đồng after the amount with a space, no decimals: `#,##0 [$₫-42A]` → `1.234.567 ₫`.
  The Number Format list's Currency and the accounting button apply đồng's formats in Vietnamese.
- **Entry** (`core::parse::parse_input_in`, `cell.set`/`range.fill` `locale: "vi-VN"`): `1.234,5`,
  `1234,5`, `12,5%`, `1.234 ₫`/`₫1.234` (currency format), `24/10/2026`, `24-10-26`, `24/10`,
  ISO dates, `14:30`, date-times. `.` is not a date separator and en-US spellings such as
  `1,234.5` or `1.5` stay text, as in Excel with vi-VN settings. The API without `locale` is en-US.
- **Formulas** (`FormulaLocale::Vi`): English function names (as Excel's Vietnamese version), `;`
  between arguments, decimal comma, `\` between array columns — the Spanish mechanism without the
  name table. Stored and saved as en-US (`SUM(A1,0.5)`).
- **Functions:** `TEXT` reads its format text the locale's way (`TEXT(A1;"#.##0,00")` →
  `1.234,50`; `numfmt::delocalize_code`), `VALUE` reads vi-VN text, `NUMBERVALUE` defaults to the
  locale's separators (Microsoft's documentation of each). Their results depend on the locale, as
  Excel's depend on the system; a workbook recalculates when the locale changes.
- **CSV (decision):** Excel reads and writes CSV with the system's list separator and number
  format. In vi-VN, Save As CSV writes `;` between fields and the displayed values (`1.234,50;24/10/2026`;
  General numbers in full precision with a decimal comma); a file read from CSV keeps its own
  separator. Opening a CSV in vi-VN reads values the vi-VN way and takes `;` as the separator when
  it appears; a file whose commas only sit between digits is one column of decimal-comma numbers;
  other comma files (en-US exports) still split at commas. Round trip in the same locale is
  lossless (tested); en-US behaviour is unchanged. Opening a vi-VN CSV in en-US reads `1.234,50`
  as text, as Excel does across locales.
- **Not yet:** a regional setting independent of the language; localized format codes in the
  Format Cells dialog (codes show en-US: `#,##0.00`); chart axis and data labels (en-US); conditions in `[<1000,5]` brackets of a
  localized TEXT code; arithmetic coercion of text (`="1,5"+1`) stays en-US; data-validation
  messages quote en-US numbers.

## Infrastructure (do first)

1. Extend the catalog from the ribbon to every user-visible string (dialogs, menus, task panes,
   status and error messages, command labels, function descriptions), with a missing-key check
   in `cargo xtask ci`. Language switching and the system-language default already exist (#31).
2. Locale-aware formulas: localized function-name tables and separators at the formula-bar
   boundary only (files keep en-US invariant names), locale number and date formats, list
   separators in CSV.
3. Text shaping and fallback: a shaping engine for grid, editor and PDF text (Arabic, Devanagari,
   Thai), system font fallback for CJK, bidirectional runs, right-to-left sheet tabs and printing;
   embedded subset fonts in PDF.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-24 | minor | NexusOffice vi-VN regional format: display, entry, đồng currency, system date formats, `FormulaLocale::Vi`, TEXT/VALUE/NUMBERVALUE per locale, CSV `;` with decimal comma (decision recorded above) |
| 2026-10-12 | minor | Format Cells looks up its tabs, labels, options and note through `Language::tr` (Fill tab keyed `Format Cells\|Fill`); dialog titles and OK/Cancel too. Labels already in a table (Number, Currency, Fill…) show translated; the rest fall back to English until the tables add them |
| 2026-10-12 | minor | Brazilian Portuguese catalog (`i18n/pt.rs`) grows from 377 to 923 entries: every key the other catalogs translate, including the dialog strings #669 starts looking up (~42%) |
| 2026-10-11 | minor | German ribbon catalog (`i18n/de.rs`, 266 labels, ~12%); `de` locale tags (de-DE, de-AT, de-CH…) resolve to German. PR #163 (localized German formulas/numbers) still open |
| 2026-10-11 | minor | Sheet Right-to-Left mirrors the grid (#146) |
| 2026-10-11 | minor | Czech (cs) ribbon table: 266 labels (~12%), menus only |
| 2026-10-10 | minor | Re-measured after #31, #104, #82, #38 merged: ribbon translated into ja, zh, ko, pt-BR and ru (266–268 labels, ~12%), system-language default, CJK/Thai/Hebrew font fallback; ~10% ready, 100–165 h |
| 2026-10-10 | major | Created: English only, no catalog, ~2,200 strings, script-support gaps, open language PRs |
