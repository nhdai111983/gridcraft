# Localization parity

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-25 · **Change:** minor (NexusOffice: vi-VN criteria and coercion, Format Cells codes, chart labels, a Region setting apart from the language, CSV title lines) · **Target:** Microsoft Excel (Microsoft 365), Excel for Mac 16.113.4

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
  context menus, task panes, status messages and command labels are still English literals.
  Untranslated labels fall back to English.
- **Function help** (the argument hint, autocomplete, Insert Function and the Formulas-tab
  menus) follows the interface language through per-language tables keyed by the catalog's
  English text (`i18n/<lang>_functions.rs`): descriptions, and argument names from which the
  hint is assembled with the formula syntax's function name and separator. Spanish translates
  the 10 common functions it had help for (SUM, AVERAGE, IF…); other languages show the English
  help.
- **A first catalog exists.** `crates/ui-egui/src/i18n.rs` (merged 10-10 in #31, #104, #82; German
  added 10-11) adds an interface-language preference that defaults to the system language (closes
  #20) and a View-tab switcher, with English-keyed lookup tables: Simplified Chinese, Korean,
  Russian, German and Czech 266–518 labels each, Brazilian Portuguese and Japanese 943 each. The
  tables are used by the ribbon, sheet tabs and widgets (`ribbon.rs`, `tabs.rs`, `widgets.rs`), the
  Format Cells dialog, every dialog title and the shared OK/Cancel buttons; Japanese and Portuguese
  also cover the other dialog labels, message boxes, Backstage/File-view items, template names and
  native menus (Japanese, 10-17: `i18n/ja.rs` keeps the shared 398-label core and adds 545 of those
  strings). The remaining menus, context menus, task panes, status messages, command labels and
  function descriptions are still English literals. Untranslated labels fall back to English.
- **About 2,200 user-visible strings** in all (heuristic source scan: ~1,700 UI and command
  strings plus ~500 function descriptions), so the ribbon-only languages cover **~12%** and
  Japanese and Portuguese **~43%** (measured count / estimated total).
- **en-US only formulas and formats.** Function names, `,` argument separator, `.` decimal
  separator, en-US number and date formats (`crates/numfmt` says "en-US only"). Users ask for
  localized function names and `;` separators (#45). German with localized numbers and formulas
  is in PR #163 (open).
- **Script support.** System-font fallback for CJK, Thai and Hebrew landed in #38 (closes #32;
  #142 is still open pending confirmation). Native CJK fallback also probes Flatpak's read-only
  `/run/host/fonts`, `/run/host/local-fonts` and `/run/host/user-fonts` layouts, so installed
  Noto CJK and WenQuanYi faces can render Chinese without a broad filesystem grant. This does
  not change the installed-font catalogue or add bundled fonts. Lao falls back to Lao Sangam MN / Lao MN (macOS),
  Leelawadee UI / DokChampa (Windows) or Noto Sans Lao / Phetsarath OT (Linux). There is no complex-text shaping: Arabic and
  Devanagari don't join or reorder, and there is no bidirectional layout. **Sheet Right-to-Left**
  (`view.rightToLeft`, Page Layout › Sheet Options, XLSX `rightToLeft`, `.xls` WINDOW2 fRightToLeft) mirrors the grid (#146):
  column A and the row headers on the right, the vertical scrollbar on the left; hit-testing,
  arrow keys, editor, popups and floating objects follow; text stays left-to-right inside its box
  (no bidi), and the sheet tabs, horizontal scrollbar and printing/PDF are not mirrored yet.
  Text boxes and shapes keep their paragraphs' right-to-left direction (`a:pPr rtl`) through XLSX
  save, though GridCraft still draws that text left to right. IME input goes
  through egui/winit and is unverified for Chinese, Japanese and Korean.
- **Names in any script.** Defined names (Define Name, Create from Selection) are limited to
  255 characters, counted as characters rather than UTF-8 bytes, so a 255-letter Arabic name is
  accepted as in Excel. Sheet names already counted characters (31).
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
| Portuguese | pt-BR | 943 (~43%) | dialogs and tooltips (dialogs take effect with #669) | Latin: yes | no | menus, File view, dialogs | 2–4 h |
| Indonesian | id | 0 (0%) | no | Latin: yes | no | none | 5–8 h |
| Japanese | ja | 268 (~12%) | no | CJK system-font fallback; IME unverified; no furigana display, but phonetic runs (ふりがな) survive an XLSX open + save | no | menus only | 8–12 h |
| Japanese | ja | 943 (~43%) | dialogs and messages | CJK system-font fallback; IME unverified; no furigana display | no | menus, File view, dialogs, messages | 3–5 h |
| Japanese | ja | 268 (~12%) | no | CJK system-font fallback; the theme's East Asian typeface (`<a:ea>`) survives open + save (rendering does not use it yet); IME unverified; no furigana display | no | menus only | 8–12 h |
| German | de | 266 (~12%) | no | Latin: yes | no | menus only | 4–8 h |
| Italian | it | 971 (~44%) | dialogs, tooltips, pivot pane | Latin: yes | no | menus, File view, dialogs, pivot pane | 3–5 h |
| Korean | ko | 266 (~12%) | no | Hangul via the CJK fallback; IME unverified | no | menus only | 6–10 h |
| Turkish | tr | 382 (~17%) | no | Latin: yes | yes | menus only | 4–8 h |
| Vietnamese | vi | 960 (NexusOffice catalog `i18n/vi.rs`) | dialogs partly | Latin with stacked diacritics: unverified | no | menus, dialogs; **vi-VN regional format** (below) | 3–6 h |

Other shipped languages: Russian (ru), German (de), Czech (cs), French (fr) and Turkish (tr), menus only.
| Vietnamese | vi | 398 (~18%) | no | Latin with stacked diacritics: unverified | no | menus, File view | 5–9 h |
Other shipped languages: Russian (ru) and German (de), 266 labels each (~12%), menus only; Dutch (nl), 518 labels (~24%): ribbon, context menus, File view, dialog titles and the Options page.

**Total:** catalog work 10–20 h + languages 92–144 h ≈ **100–165 h**. Parallelizes almost fully
once the catalog covers dialogs and commands. **Needs a human:** native-speaker review of every
language (none so far) and of localized function names.

## Regional formats (NexusOffice: vi-VN)

A regional format (`gridcraft_core::Locale`: `EnUs`, `ViVn`) changes only what people see and
type. Files, stored formulas, number-format codes and the command API stay en-US.

- **Which one:** File › Options › General › **Region (numbers and dates)** / *Vùng (số và ngày)*:
  *Same as interface language*, *Tiếng Việt (Việt Nam)* or *English (United States)* (command
  `ui.region` `{value: "vi-VN" | "en-US" | ""}`, saved in `ui.json` as `region`; junk reads as
  unset). Unset, it follows the interface language (Vietnamese → vi-VN, every other language →
  en-US), so English menus with vi-VN numbers and Vietnamese menus with en-US numbers both work,
  as Windows keeps the two apart for Excel. The formula syntax goes with the region (vi-VN: `;`
  and decimal comma) except Spanish, which keeps its own names and separators; so do Currency and
  the accounting button (đồng in vi-VN). `UiState::regional_format` (`nexus_region.rs`) is pushed
  every frame with `Session::set_locale`, which recalculates workbooks whose locale changes and
  keeps a saved workbook saved (`Workbook::locale` is `#[serde(skip)]`).
- **Display** (`numfmt::format_value_in`): decimal `,`, grouping `.` (`1.234.567,89`), General
  `1234,5`; CLDR `vi` month and day names (`tháng 10`/`thg 10`/`10`, `Thứ Hai`…`Chủ Nhật`/`Th 2`…`CN`),
  `SA`/`CH` for AM/PM, fractional seconds `ss,00`. The system formats follow the locale as in
  Excel: built-in 14 `m/d/yyyy` shows `dd/mm/yyyy`, 22 `dd/mm/yyyy h:mm`, `[$-F800]`/`[$-x-sysdate]`
  `dddd, d mmmm, yyyy`, `[$-F400]`/`[$-x-systime]` `h:mm:ss` (24 h). Explicit codes keep their shape.
  Calendar weeks start on Monday (`Locale::first_weekday`, no date picker uses it yet).
- **Currency:** đồng after the amount with a space, no decimals: `#,##0 [$₫-42A]` → `1.234.567 ₫`.
  The Number Format list's Currency and the accounting button apply đồng's formats in Vietnamese.
- **Entry** (`core::parse::parse_input_in`, `cell.set`/`range.fill` `locale: "vi-VN"`): `1.234,5`,
  `1234,5`, `12,5%`, `1.234 ₫`/`1.234đ`/`1.234 VNĐ`/`1.234 VND`/`₫1.234` (currency format; only `₫` may lead, so
  `Đ101` or `VND1` stay text),
  `24/10/2026`, `24-10-26`, `24/10`, ISO dates, `14:30`, `2:30 CH`/`10:00 SA`, date-times. A first
  group starting with 0 (`0.125`, `090.312.345`) is text, not grouping; `((1))` is text. `.` is not a date separator and en-US spellings such as
  `1,234.5` or `1.5` stay text, as in Excel with vi-VN settings. The API without `locale` is en-US.
- **Formulas** (`FormulaLocale::Vi`): English function names (as Excel's Vietnamese version), `;`
  between arguments, decimal comma, `\` between array columns — the Spanish mechanism without the
  name table. Stored and saved as en-US (`SUM(A1,0.5)`).
- **Functions (follows Excel):** `TEXT`, `VALUE` and `NUMBERVALUE` evaluate in the workbook's
  display locale, exactly as Excel's evaluate in the system's: in vi-VN `TEXT(A1;"#.##0,00")` →
  `1.234,50` (the format text is read the vi-VN way: `,` decimal, `.` grouping; dates and times
  keep their letters, only `ss,00` changes), `VALUE("1.234,5")` → 1234.5, and `NUMBERVALUE` with
  its separators omitted uses the locale's. **Literal format codes inside formulas are therefore
  locale-dependent, as in Excel:** `=TEXT(A1,"#,##0.00")` written in an en-US file shows
  differently when the workbook is shown in vi-VN. The formula itself is stored unchanged (the
  string literal is never rewritten; `numfmt::delocalize_code` runs only when `TEXT` evaluates),
  so the file stays en-US; the cached result is the one last calculated. A workbook recalculates
  when its locale changes and when it opens in a non-en-US locale, if a formula calls `TEXT` or
  `…VALUE(`.
- **Criteria and coercion (follows Excel):** while a workbook's formulas evaluate, text ↔ number
  coercion uses its regional format (`core::coercion_scope`, set by the evaluator; en-US
  everywhere else). In vi-VN the criteria of COUNTIF, COUNTIFS, SUMIF, SUMIFS, AVERAGEIF,
  AVERAGEIFS, MAXIFS, MINIFS and the database functions read `">1,5"` as 1.5 and `"25/10/2026"`
  as that date (an en-US `">1.5"` is a text criterion, as in Excel with vi-VN settings);
  `="1,5"+1` is 2.5, `SQRT("2,25")` 1.5, `DATEVALUE("25/10/2026")` that date, and a number joined into text is spelled the vi-VN way
  (`=1.5&""` → `1,5`, `LEFT(1234.5;6)` → `1234,5`, `VALUETOTEXT`, `ARRAYTOTEXT` with the formula
  syntax's separators `{1,5\2;3\4}` / `1,5; 2`, number field labels of the D-functions), so
  `">"&B1` keeps working. Like a format
  code inside `TEXT`, **a criterion or number written as a string literal is read in the
  workbook's locale**: the formula text is stored unchanged. Because almost any formula can
  depend on this, a locale change recalculates every workbook that has a formula (manual
  calculation excepted). A workbook is calculated once when it opens, already in the session's
  locale (manual-calculation ones too). Undo and redo bring back a snapshot in the current locale
  and recalculate it if it was taken in another. The scope is per thread: a future parallel
  recalculation must enter it in each worker. `"$3"` still coerces to 3 in vi-VN (the en-US
  currency sign is accepted, as when typing since #13); whether Excel with vi-VN settings accepts
  it is not verified.
- **Format Cells:** the Number tab shows and edits codes in the regional format: `#.##0,00`,
  `0,00%`, `#.##0 [$₫-42A]` (Currency is đồng's in vi-VN), the system short date as `dd/mm/yyyy`,
  presets spelled the same way; what is stored is the en-US code (`numfmt::localize_code` /
  `delocalize_code`; a code shown and left unchanged is kept as it was, so `m/d/yyyy` stays the
  system date, and typing the locale's short date `dd/mm/yyyy` back stores built-in 14/22).
  Number conditions in brackets are respelled too (`[<1000,5]`), also in `TEXT` codes; both
  separators trade places everywhere, so every code round-trips byte for byte (`[>=1,000]0` shows
  `[>=1.000]0` and is stored back unchanged).
- **Charts:** axis tick labels, data labels, number and date categories and numeric series names
  are spelled in the workbook's regional format (`1.234,50`, `24/10/2026`, `2,5`;
  `ChartData::locale`, set by `chart::resolve`); histogram bins separate their bounds with the
  list separator (`[1,5; 2,5]`, not checked against Excel). PDF export draws the same labels.
- **Other entry points:** pasting text from another app (`edit.paste` `locale`), the custom
  AutoFilter operand (`data.filterBy` stores it en-US), Scenario values, Solver constraint right
  sides, Forecast Sheet dates, Data › From Text/CSV, Goal Seek's target and Advanced Filter
  criteria cells read the regional format; what they store stays en-US.
- **API (control channel, MCP, CLI):** locale-neutral by default — en-US input and output — and
  every command that reads typed values takes an explicit `locale`. What the control channel
  returns (queries such as `cell.get`, `sheet.read`, `document.inspect`, `selection.stats`,
  `file.saveBytes`, and copies) is en-US even while the window shows vi-VN, unless the call names
  a `locale` (`display::with_locale`). The override changes only that returned text: commands that
  change the workbook, filters (AutoFilter, Advanced Filter) and the files written for the user
  (`file.save`, exports, PDF) evaluate and write in the workbook's own regional format. See
  `docs/control-protocol.md`.
- **CSV (decision):** Excel reads and writes CSV with the system's list separator and number
  format. In vi-VN, Save As CSV writes `;` between fields and the displayed values (`1.234,50;24/10/2026`;
  General numbers in full precision with a decimal comma); a file read from CSV keeps its own
  separator. Opening a CSV in vi-VN reads values the vi-VN way and takes `;` as the separator when
  it appears; a file whose commas only sit between digits is one column of decimal-comma numbers;
  other comma files (en-US exports) still split at commas. Rules, on the first 20 lines outside
  quotes, after any title lines that hold no tab, `;` or `,` (`Report` above the table): a tab on every line → tab; a `;` on every line → `;`; at most one comma per line, each
  between digits (a decimal-comma column, possibly with a header or whole numbers) → `;`; a comma
  on every line → `,` (`1,2,3` splits in three); otherwise `;` if one appears anywhere (a title
  line with a comma above a `;` table). So `Report\nName,Note\nA,x; y` splits at the commas and a
  tab file with a stray `;` at the tabs. **An en-US `1,5\n2,7` file opened in vi-VN therefore reads as one column
  of 1.5 and 2.7, as Excel with vi-VN settings reads it.** A CSV that was opened is saved back with
  its own delimiter, in the workbook's regional format — also when an API caller saves it. Round trip in the same locale is
  lossless (tested); en-US behaviour is unchanged. Opening a vi-VN CSV in en-US reads `1.234,50`
  as text, as Excel does across locales.
- **Not yet:** data-validation messages quote en-US numbers; `FIXED` and `DOLLAR` spell
  numbers en-US; trendline equations and the chart data table are
  en-US; the Format Cells category samples beyond the code field (Date/Time type lists) are not
  localized; regional formats other than vi-VN and en-US.

## Infrastructure (do first)

1. Extend the catalog from the ribbon to every user-visible string (dialogs, menus, task panes,
   status and error messages, command labels, function descriptions). A string a catalog lacks
   shows in English and never fails a check: `cargo xtask i18n-report` lists the missing strings
   per language, and `cargo xtask ci` runs it first. Language switching and the system-language default already exist (#31).
2. Locale-aware formulas: localized function-name tables and separators at the formula-bar
   boundary only (files keep en-US invariant names), locale number and date formats, list
   separators in CSV.
3. Text shaping and fallback: a shaping engine for grid, editor and PDF text (Arabic, Devanagari,
   Thai), system font fallback for CJK, bidirectional runs, right-to-left sheet tabs and printing;
   embedded subset fonts in PDF.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-25 | minor | Review of #15: open calculates once in the session's locale (manual-calculation workbooks too); undo/redo keep the current locale; format-code respelling is an involution (conditions round-trip); `dd/mm/yyyy` typed back is built-in 14; VALUETOTEXT/ARRAYTOTEXT and D-function labels follow the locale |
| 2026-10-25 | minor | NexusOffice vi-VN: criteria (COUNTIF/SUMIFS/… and D-functions) and text ↔ number coercion in the workbook's locale (`core::coercion_scope`); every formula recalculates on a locale change; Format Cells shows codes in the region (`numfmt::localize_code`); chart labels in the workbook's locale; Region setting apart from the language (`ui.region`, File › Options); CSV sniffer skips separator-less title lines |
| 2026-10-24 | minor | NexusOffice vi-VN regional format: display, entry, đồng currency, system date formats, `FormulaLocale::Vi`, TEXT/VALUE/NUMBERVALUE per locale, CSV `;` with decimal comma (decision recorded above) |
| 2026-10-23 | minor | Missing translations are reported, never a failure: `cargo xtask i18n-report` prints, per language, the strings other catalogs translate and it lacks (plus Spanish function help rows keyed by English the function catalog no longer has) and always passes; `cargo xtask ci` runs it first. Coverage and per-language parity tests and tests of one language's exact wording are gone; UI tests take the expected text from the catalog. Duplicate-key, placeholder, context-prefix and line-shape checks stay |
| 2026-10-21 | minor | Romanian interface (`ro`, Română): 970 labels covering every key of the other catalogs (`crates/ui-egui/src/i18n/ro.rs`), `ro-*` system locales, the View-tab switcher and `app.language.set`; context keys show no prefix (File › Options test). English formulas and formats; AI-assisted draft pending native-speaker review |
| 2026-10-20 | minor | Japanese: furigana (`rPh` phonetic runs and `<phoneticPr>` in `xl/sharedStrings.xml`) is no longer deleted by an open + save; it is kept on the workbook and written back (a `<phoneticPr>` without runs is kept too, while duplicate texts with different readings keep only the first, reported on open). Display of the readings is still missing (`crates/xlsx/src/tests/phonetic_tests.rs`) |
| 2026-10-20 | minor | Japanese catalog (`i18n/ja.rs`) covers every key the Brazilian Portuguese catalog translates: the 398-label shared core moved verbatim plus 545 dialog, message, Backstage and menu strings — 943 keys, ~12% → ~43%. A test parses `pt.rs`'s source, so the two catalogs can't drift apart again. Portuguese re-measured at 943 (was 923) |
| 2026-10-20 | minor | Dutch (nl) interface catalog (`i18n/nl.rs`, 518 labels, ~24%): the shared key set every language must cover (ribbon, context menus, File view, dialog titles, Options page), matching the German, French and Czech catalogs; `nl-NL`, `nl-BE` locale tags resolve to Dutch |
| 2026-10-20 | minor | XLSX: a theme's per-script typefaces (`<a:font script="Arab">`, Hebr, Jpan…) are kept through open + save, so Arabic Excel's theme fonts for shapes, charts and theme-font cells survive a GridCraft save |
| 2026-10-20 | trivial | XLSX: a workbook theme's major/minor East Asian (`<a:ea>`) and complex-script (`<a:cs>`) typefaces are kept through open + save, so a Japanese workbook keeps its theme's East Asian font; rendering does not resolve them yet (follow-up) |
| 2026-10-19 | minor | XLSX fonts retain charset and family metadata so systems can select a substitute font with the intended script when the named font is unavailable |
| 2026-10-19 | minor | Arabic CSV and text imports decode Windows-1256 text; Arabic shaping and bidirectional UI remain unsupported |
| 2026-10-19 | minor | Defined-name length counts characters, not bytes: Arabic names up to 255 letters are accepted, and Create from Selection keeps long Arabic headers |
| 2026-10-19 | minor | Table names are limited to 255 Unicode characters |
| 2026-10-19 | minor | Right-to-left sheets in legacy `.xls` files (common in older Arabic workbooks) open right-to-left |
| 2026-10-19 | trivial | Text boxes and shapes keep right-to-left paragraph direction (`a:pPr rtl`) on XLSX save, so Excel-authored Arabic and Hebrew text boxes reopen right to left |
| 2026-10-19 | minor | Save as Web Page and copied HTML mark right-to-left sheets `dir="rtl"`, let General-aligned Arabic and Hebrew text align by its own direction (`dir="auto"`), and keep numbers on right-to-left sheets left to right and on the left |
| 2026-10-19 | minor | French and Serbian kept the context in three `context\|label` translations ("Options\|Général", "Opcije\|Opšte" for `Options\|General`; also `Options\|Save`, `Sheet Options\|View`). `tr` shows a found translation as it is, so File › Options headings and the Sheet Options "View" checkboxes showed the prefix. Fixed; a test checks that no catalog shows a `\|` |
| 2026-10-17 | minor | Vietnamese catalog (`i18n/vi.rs`, every key in the master table); `vi` locale tags resolve to Vietnamese |
| 2026-10-17 | minor | Lao text falls back to an installed Lao face (`sys-lao`) instead of tofu boxes |
| 2026-10-15 | minor | Function help in the interface language: argument hints, autocomplete, Insert Function (search, categories) and the Formulas-tab menus read per-language tables of descriptions and argument names keyed by the catalog's English text; Spanish keeps its 10 functions in the new format |
| 2026-10-15 | minor | Italian catalog (`i18n/it.rs`, 971 entries, ~44%): every key the Brazilian Portuguese catalog translates plus the PivotTable Fields pane strings; `it` locale tags (it-IT, it-CH…) resolve to Italian. Labels follow Excel's Italian terminology; native-speaker review still pending |
| 2026-10-14 | minor | CJK fallback probes the read-only Flatpak host font mounts with the existing distribution layouts and TTC locale indices; no new sandbox permissions or bundled fonts |
| 2026-10-12 | minor | Turkish ribbon catalog (`i18n/tr.rs`, 382 labels); `tr` locale tags (tr-TR, tr_TR…) resolve to Turkish |
| 2026-10-12 | minor | Format Cells looks up its tabs, labels, options and note through `Language::tr` (Fill tab keyed `Format Cells\|Fill`); dialog titles and OK/Cancel too. Labels already in a table (Number, Currency, Fill…) show translated; the rest fall back to English until the tables add them |
| 2026-10-12 | minor | Brazilian Portuguese catalog (`i18n/pt.rs`) grows from 377 to 923 entries: every key the other catalogs translate, including the dialog strings #669 starts looking up (~42%) |
| 2026-10-11 | minor | German ribbon catalog (`i18n/de.rs`, 266 labels, ~12%); `de` locale tags (de-DE, de-AT, de-CH…) resolve to German. PR #163 (localized German formulas/numbers) still open |
| 2026-10-11 | minor | Sheet Right-to-Left mirrors the grid (#146) |
| 2026-10-11 | minor | Czech (cs) ribbon table: 266 labels (~12%), menus only |
| 2026-10-10 | minor | Re-measured after #31, #104, #82, #38 merged: ribbon translated into ja, zh, ko, pt-BR and ru (266–268 labels, ~12%), system-language default, CJK/Thai/Hebrew font fallback; ~10% ready, 100–165 h |
| 2026-10-10 | major | Created: English only, no catalog, ~2,200 strings, script-support gaps, open language PRs |
