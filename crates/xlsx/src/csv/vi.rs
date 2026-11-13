//! NexusOffice: CSV delimiter sniffing for regional formats (vi-VN, whose list separator is `;`
//! and decimal separator `,`). Kept in its own file, on top of the shared sniffer that
//! [`super`] re-exports from craft-csv, so upstream changes to `csv.rs` don't collide with it.

/// [`super::sniff_delimiter`] for a file written in a regional format. Where the list separator is `;`
/// (vi-VN, whose decimal separator is `,`), Excel reads CSV with `;`. Looking at the first lines
/// (outside quotes, after any title lines that hold no tab, `;` or `,`), in this order:
/// 1. a tab on every line: tab;
/// 2. a `;` on every line: `;`, even when the decimal commas outnumber it;
/// 3. at most one comma per line, each between two digits (a column of `1,5`, maybe with a
///    header or whole numbers): decimal commas, `;` (one column unless `;` appears);
/// 4. a comma on every line (an en-US export, `1,2,3`): `,`;
/// 5. no consistent separator: `;` if one appears anywhere (a title line above a `;` table),
///    else what [`super::sniff_delimiter`] says.
pub fn sniff_delimiter_in(text: &str, quote: char, locale: gridcraft_core::Locale) -> u8 {
    let sniffed = super::sniff_delimiter(text, quote);
    if locale.list_separator() != ';' {
        return sniffed;
    }
    // Per non-blank line (the first 20, as `sniff_delimiter` reads): tabs, `;`, `,`, and whether
    // every `,` sits between two digits.
    #[derive(Clone, Copy)]
    struct Line {
        tabs: usize,
        semis: usize,
        commas: usize,
        digit_commas: bool,
    }
    let fresh = Line { tabs: 0, semis: 0, commas: 0, digit_commas: true };
    let mut lines: Vec<Line> = Vec::new();
    let mut cur = fresh;
    let mut blank = true;
    let mut in_q = false;
    let mut prev: Option<char> = None;
    let mut chars = text.chars().take(64 * 1024).peekable();
    while let Some(ch) = chars.next() {
        if !in_q && ch == '\n' {
            if !blank {
                lines.push(cur);
            }
            cur = fresh;
            blank = true;
            prev = None;
            if lines.len() >= 20 {
                break;
            }
            continue;
        }
        if ch != '\r' {
            blank = false;
        }
        if ch == quote {
            in_q = !in_q;
        } else if !in_q {
            match ch {
                '\t' => cur.tabs += 1,
                ';' => cur.semis += 1,
                ',' => {
                    cur.commas += 1;
                    if !(prev.is_some_and(|c| c.is_ascii_digit()) && chars.peek().is_some_and(char::is_ascii_digit)) {
                        cur.digit_commas = false;
                    }
                }
                _ => {}
            }
        }
        prev = Some(ch);
    }
    if !blank && lines.len() < 20 {
        lines.push(cur);
    }
    // Title lines above the table (`Report`, no separator at all) say nothing about it: they
    // would fail every "on every line" rule and leave a stray `;` in a text cell to decide.
    let first = lines.iter().position(|l| l.tabs + l.semis + l.commas > 0).unwrap_or(0);
    let lines = lines.get(first..).unwrap_or(&[]);
    if lines.is_empty() {
        return sniffed;
    }
    let every = |f: fn(&Line) -> bool| lines.iter().all(f);
    if every(|l| l.tabs > 0) {
        return b'\t';
    }
    if every(|l| l.semis > 0) {
        return b';';
    }
    if lines.iter().any(|l| l.commas > 0) && every(|l| l.commas <= 1 && l.digit_commas) {
        return b';';
    }
    if every(|l| l.commas > 0) {
        return b',';
    }
    if lines.iter().any(|l| l.semis > 0) {
        return b';';
    }
    sniffed
}
