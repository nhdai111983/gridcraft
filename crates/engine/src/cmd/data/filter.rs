use gridcraft_core::{TextKey, TextSet, fold_key, text_eq_prefolded, text_key};

use super::sort::sort_key;
use super::{
    AutoFilter, CellRef, EngineError, FilterCriterion, Json, Ordering, RangeRef, Result, Session, Sheet, Value, Workbook, bad, bool_param, col_param,
    current_region, data_target, edit, json,
};

// ---------------------------------------------------------------- filter

pub(super) fn toggle_filter(s: &mut Session, p: &Json) -> Result<Json> {
    let (sheet, r) = data_target(s, p, "data.filter")?;
    crate::cmd::refuse_unless_allowed(s, sheet, "Filtering", |p| p.autofilter)?;
    // The active cell belongs to the active sheet: on another sheet the table is the one at `range`.
    let at = if sheet == s.doc()?.wb.active_sheet() { s.doc()?.selection.active } else { r.start };
    let has = s.doc()?.wb.sheet(sheet).is_some_and(|sh| sh.autofilter.is_some());
    let in_table = s.doc()?.wb.sheet(sheet).and_then(|sh| sh.table_at(at).map(|t| t.name.clone()));
    if let Some(name) = in_table {
        return edit(s, |cx| {
            let sh = cx.sheet_mut(sheet)?;
            if let Some(t) = sh.tables.iter_mut().find(|t| t.name == name) {
                t.filter_button = !t.filter_button;
                if !t.filter_button {
                    t.filter.clear();
                }
            }
            Ok(Json::Null)
        });
    }
    if has {
        return edit(s, |cx| {
            let sh = cx.sheet_mut(sheet)?;
            // Reveal only rows the filter hid; rows hidden by hand or by a collapsed group stay hidden.
            let filtered: Vec<u32> = sh.rows.keys().copied().filter(|r| sh.is_row_filtered(*r)).collect();
            sh.autofilter = None;
            for r in filtered {
                if let Some(info) = sh.rows.get_mut(&r) {
                    info.hidden = false;
                }
            }
            cx.structural = true;
            Ok(json!({"filter": false}))
        });
    }
    install_filter(s, p)
}

/// Turn a sheet AutoFilter on over `p.range`, or over the data range of the selection.
fn install_filter(s: &mut Session, p: &Json) -> Result<Json> {
    let (sheet, mut r) = data_target(s, p, "data.filter")?;
    // A selected header row filters the data below it (#201).
    if p.get("range").is_none() && r.height() == 1 {
        let d = s.doc()?;
        let sh = d.wb.active().ok_or(EngineError::NoDocument)?;
        let below = current_region(sh, r.start).end.row;
        if below > r.end.row {
            r = RangeRef::new(r.start, CellRef::new(below, r.end.col));
        }
    }
    edit(s, |cx| {
        cx.sheet_mut(sheet)?.autofilter = Some(AutoFilter { range: r, criteria: vec![] });
        Ok(json!({"filter": true, "range": r.a1()}))
    })
}

pub(super) fn filter_by(s: &mut Session, p: &Json) -> Result<Json> {
    let sheet = s.doc()?.wb.active_sheet();
    crate::cmd::refuse_unless_allowed(s, sheet, "Filtering", |p| p.autofilter)?;
    let Some(sh) = s.doc()?.wb.sheet(sheet) else { return Err(EngineError::NoDocument) };
    let af = match &sh.autofilter {
        Some(a) => a.clone(),
        None => {
            install_filter(s, &json!({}))?;
            s.doc()?.wb.sheet(sheet).and_then(|sh| sh.autofilter.clone()).ok_or_else(|| EngineError::Other("no filter".into()))?
        }
    };
    let col = match col_param(p.get("column"), af.range.start.col) {
        Some(c) => c,
        // A number too large to be a column is outside the filter range, not a missing parameter.
        None if p.get("column").is_some_and(|v| v.as_u64().is_some()) => return Err(bad("data.filterBy", "column outside the filter range")),
        None => return Err(bad("data.filterBy", "missing `column`")),
    };
    if col < af.range.start.col || col > af.range.end.col {
        return Err(bad("data.filterBy", "column outside the filter range"));
    }
    let off = col - af.range.start.col;
    let crit = if bool_param(p, "clear").unwrap_or(false) {
        None
    } else if let Some(vals) = p.get("values").and_then(Json::as_array) {
        Some(FilterCriterion::Values {
            values: vals.iter().map(crate::cmd::edit::json_to_input).collect(),
            blanks: bool_param(p, "blanks").unwrap_or(false),
        })
    } else if let Some(c) = p.get("custom") {
        let a =
            (c.get("op").and_then(Json::as_str).unwrap_or("=").to_string(), c.get("value").map(crate::cmd::edit::json_to_input).unwrap_or_default());
        let b =
            c.get("op2").and_then(Json::as_str).map(|o| (o.to_string(), c.get("value2").map(crate::cmd::edit::json_to_input).unwrap_or_default()));
        Some(FilterCriterion::Custom { a, b, and: c.get("and").and_then(Json::as_bool).unwrap_or(true) })
    } else if let Some(t) = p.get("top") {
        Some(FilterCriterion::Top10 {
            bottom: t.get("bottom").and_then(Json::as_bool).unwrap_or(false),
            percent: t.get("percent").and_then(Json::as_bool).unwrap_or(false),
            count: t.get("count").and_then(Json::as_u64).unwrap_or(10) as u32,
        })
    } else {
        bool_param(p, "aboveAverage").map(FilterCriterion::AboveAverage)
    };
    edit(s, |cx| {
        let sh = cx.sheet_mut(sheet)?;
        if let Some(a) = sh.autofilter.as_mut() {
            a.criteria.retain(|(o, _)| *o != off);
            if let Some(c) = crit.clone() {
                a.criteria.push((off, c));
            }
        }
        apply_filter(&mut cx.wb, sheet);
        cx.structural = true;
        Ok(Json::Null)
    })?;
    let hidden = s.doc()?.wb.sheet(sheet).map(|sh| sh.rows.values().filter(|r| r.hidden).count()).unwrap_or(0);
    Ok(json!({"hiddenRows": hidden}))
}

/// Does a value pass a custom comparison (`>`, `=*abc*`, `<>x`…)? `folded` is `fold_key(crit)`,
/// computed once per criterion for the equality and begins/ends/contains tests. Only what `op`
/// needs is computed: text is ordered (a key per side) for `<`, `<=`, `>` and `>=` alone.
fn custom_ok(v: &Value, text: &str, op: &str, crit: &str, folded: &str) -> bool {
    let num = gridcraft_core::parse::parse_number_text(crit);
    let nums = v.as_f64().zip(num);
    let cmp = || match (v.as_f64(), num) {
        (Some(a), Some(b)) => a.partial_cmp(&b),
        // A numeric ordering comparison only matches numbers; blanks and text never do.
        (None, Some(_)) if matches!(op, "<" | "<=" | ">" | ">=") => None,
        _ => Some(gridcraft_core::compare_text(text, crit)),
    };
    // `compare_text(text, crit) == Equal` exactly when `text_eq(text, crit)`.
    let eq = || match nums {
        Some((a, b)) => a.partial_cmp(&b) == Some(Ordering::Equal),
        None => text_eq_prefolded(text, folded),
    };
    let wild = || crate::cmd::edit::wildcard(&text.to_lowercase(), &crit.to_lowercase());
    match op {
        "=" => {
            if crit.contains(['*', '?']) {
                wild()
            } else {
                eq()
            }
        }
        "<>" => {
            if crit.contains(['*', '?']) {
                !wild()
            } else {
                !eq()
            }
        }
        ">" => cmp() == Some(Ordering::Greater),
        "<" => cmp() == Some(Ordering::Less),
        ">=" => matches!(cmp(), Some(Ordering::Greater | Ordering::Equal)),
        "<=" => matches!(cmp(), Some(Ordering::Less | Ordering::Equal)),
        // Case- and encoding-blind, matching whole letters (`e` is not in `Nguyễn`).
        "beginsWith" => gridcraft_core::starts_with_prefolded(text, folded),
        "endsWith" => gridcraft_core::ends_with_prefolded(text, folded),
        "contains" => gridcraft_core::contains_prefolded(text, folded),
        "notContains" => !gridcraft_core::contains_prefolded(text, folded),
        _ => true,
    }
}

/// The filter's "(Blanks)" group: truly empty cells and zero-length text such as a `=""` result.
fn filter_blank(v: &Value) -> bool {
    v.is_empty() || matches!(v, Value::Text(t) if t.is_empty())
}

/// Excel grows an AutoFilter range when a value is entered in the row right below it, in one of its columns (#2342).
/// A table's own filter is left to the table, and a row typed after a blank row does not count.
/// A formula counts before it is calculated: this runs inside the edit, ahead of the recalc.
pub(crate) fn grow_autofilter(sh: &mut Sheet, at: CellRef) {
    if !sh.cells.get(at).is_some_and(|c| !filter_blank(&c.value) || c.formula.is_some()) {
        return;
    }
    let Some(af) = sh.autofilter.as_ref() else { return };
    let r = af.range;
    if r.end.row.checked_add(1) != Some(at.row) || at.col < r.start.col || at.col > r.end.col || sh.tables.iter().any(|t| t.range == r) {
        return;
    }
    let grown = RangeRef::new(r.start, CellRef::new(at.row, r.end.col));
    if let Some(af) = sh.autofilter.as_mut() {
        af.range = grown;
    }
}

/// Value lists folded once into a set (a fold per cell, not a comparison per cell and value).
fn folded_value_sets(af: &AutoFilter) -> Vec<Option<TextSet>> {
    af.criteria
        .iter()
        .map(|(_, c)| match c {
            FilterCriterion::Values { values, .. } => Some(TextSet::new(values)),
            _ => None,
        })
        .collect()
}

/// Custom criteria folded once (for equality and begins/ends/contains), not once per row.
fn folded_custom_criteria(af: &AutoFilter) -> Vec<(String, String)> {
    af.criteria
        .iter()
        .map(|(_, c)| match c {
            FilterCriterion::Custom { a, b, .. } => (fold_key(&a.1), b.as_ref().map(|b| fold_key(&b.1)).unwrap_or_default()),
            _ => (String::new(), String::new()),
        })
        .collect()
}

/// Hides rows of the autofilter range that fail any criterion.
pub(crate) fn apply_filter(wb: &mut Workbook, sheet: usize) {
    let Some(sh) = wb.sheet(sheet) else { return };
    let Some(af) = sh.autofilter.clone() else { return };
    let r = af.range;
    let mut hide = Vec::new();
    // Precompute per-criterion top10 threshold and average once, not per row.
    let mut col_stats: std::collections::HashMap<usize, (Option<f64>, f64)> = std::collections::HashMap::new();
    for (i, (off, c)) in af.criteria.iter().enumerate() {
        if matches!(c, FilterCriterion::Top10 { .. } | FilterCriterion::AboveAverage(_)) {
            let col = r.start.col + off;
            let mut nums: Vec<f64> = (r.start.row + 1..=r.end.row).filter_map(|row| sh.value(CellRef::new(row, col)).as_f64()).collect();
            let avg = if nums.is_empty() { 0.0 } else { nums.iter().sum::<f64>() / nums.len() as f64 };
            let thr = match c {
                FilterCriterion::Top10 { bottom, percent, count } => {
                    nums.sort_by(|a, b| if *bottom { a.partial_cmp(b) } else { b.partial_cmp(a) }.unwrap_or(Ordering::Equal));
                    let k = if *percent { ((nums.len() as f64) * (*count as f64) / 100.0).ceil() as usize } else { *count as usize };
                    let k = k.clamp(1, nums.len().max(1));
                    nums.get(k - 1).copied()
                }
                _ => None,
            };
            col_stats.insert(i, (thr, avg));
        }
    }
    let value_sets = folded_value_sets(&af);
    let folded = folded_custom_criteria(&af);
    for row in r.start.row + 1..=r.end.row {
        let mut visible = true;
        for (i, (off, crit)) in af.criteria.iter().enumerate() {
            let c = CellRef::new(row, r.start.col + off);
            let v = sh.value(c);
            let text = crate::display::cell_text(wb, sh, c);
            let ok = match crit {
                FilterCriterion::Values { blanks, .. } => {
                    if filter_blank(&v) {
                        *blanks
                    } else {
                        // Always `Some`: built from these criteria above.
                        value_sets.get(i).and_then(Option::as_ref).is_some_and(|set| set.contains(&text) || set.contains(&v.display()))
                    }
                }
                FilterCriterion::Custom { a, b, and } => {
                    let (fa, fb) = folded.get(i).map_or(("", ""), |(x, y)| (x.as_str(), y.as_str()));
                    let x = custom_ok(&v, &text, &a.0, &a.1, fa);
                    match b {
                        Some(b) => {
                            let y = custom_ok(&v, &text, &b.0, &b.1, fb);
                            if *and { x && y } else { x || y }
                        }
                        None => x,
                    }
                }
                FilterCriterion::Top10 { bottom, .. } => match (v.as_f64(), col_stats.get(&i)) {
                    (Some(n), Some((thr, _))) => {
                        let thr = thr.unwrap_or(n);
                        if *bottom { n <= thr } else { n >= thr }
                    }
                    _ => false,
                },
                FilterCriterion::AboveAverage(above) => match (v.as_f64(), col_stats.get(&i)) {
                    (Some(n), Some((_, avg))) => {
                        if *above {
                            n > *avg
                        } else {
                            n < *avg
                        }
                    }
                    _ => false,
                },
                FilterCriterion::FillColor(col) => wb.styles.get(sh.style_id(c)).fill.fg == *col,
                FilterCriterion::FontColor(col) => wb.styles.get(sh.style_id(c)).font.color == *col,
                // Icons are not evaluated here: the row keeps its current visibility.
                FilterCriterion::Icon { .. } => !sh.is_row_hidden(row),
            };
            if !ok {
                visible = false;
                break;
            }
        }
        hide.push((row, !visible));
    }
    let Some(shm) = wb.sheet_mut(sheet) else { return };
    for (row, h) in hide {
        if h {
            shm.rows.entry(row).or_default().hidden = true;
        } else if let Some(e) = shm.rows.get_mut(&row) {
            e.hidden = false;
        }
    }
}

pub(super) fn clear_filter(s: &mut Session, _: &Json) -> Result<Json> {
    let sheet = s.doc()?.wb.active_sheet();
    edit(s, |cx| {
        let sh = cx.sheet_mut(sheet)?;
        if let Some(a) = sh.autofilter.as_mut() {
            a.criteria.clear();
        }
        apply_filter(&mut cx.wb, sheet);
        cx.structural = true;
        Ok(Json::Null)
    })
}

pub(super) fn reapply(s: &mut Session, _: &Json) -> Result<Json> {
    let sheet = s.doc()?.wb.active_sheet();
    edit(s, |cx| {
        apply_filter(&mut cx.wb, sheet);
        cx.structural = true;
        Ok(Json::Null)
    })
}

/// Distinct display values of a filter column (for the dropdown list).
pub fn filter_values(wb: &Workbook, sheet: usize, col: u32) -> Vec<(String, bool)> {
    let Some(sh) = wb.sheet(sheet) else { return vec![] };
    let Some(af) = &sh.autofilter else { return vec![] };
    let off = col.saturating_sub(af.range.start.col);
    let selected: Option<&FilterCriterion> = af.criteria.iter().find(|(o, _)| *o == off).map(|(_, c)| c);
    // The chosen values folded once (the UI calls this every frame while the menu is open).
    let chosen: Option<(TextSet, bool)> = match selected {
        Some(FilterCriterion::Values { values, blanks }) => Some((TextSet::new(values), *blanks)),
        _ => None,
    };
    // One entry per group key, showing the first text met. Text is grouped by `fold_key`, so
    // "Abc", "abc" and an NFD spelling are one entry; other values by their key and shown text.
    let mut seen: std::collections::HashSet<(u8, i64, String, String)> = std::collections::HashSet::new();
    let mut entries: Vec<((u8, i64, String, String), Option<TextKey>, String, bool)> = Vec::new();
    for row in af.range.start.row + 1..=af.range.end.row.min(af.range.start.row + 1_000_000) {
        let c = CellRef::new(row, col);
        let v = sh.value(c);
        let blank = filter_blank(&v);
        let t = if blank { "(Blanks)".to_string() } else { crate::display::cell_text(wb, sh, c) };
        let on = match &chosen {
            Some((set, blanks)) => {
                if blank {
                    *blanks
                } else {
                    set.contains(&t)
                }
            }
            None => true,
        };
        let v = if blank { Value::Empty } else { v };
        let (class, num, k) = sort_key(&v);
        let key = if v.is_text() { (class, num, fold_key(&t), String::new()) } else { (class, num, k, t.clone()) };
        if seen.insert(key.clone()) {
            // The sort key is built once per entry, not per comparison (the list is rebuilt
            // every frame while the filter menu is open).
            let tk = v.is_text().then(|| text_key(&t));
            entries.push((key, tk, t, on));
        }
    }
    // Numbers, then text in the Root order, then booleans, errors and blanks. Text entries are
    // ordered by their `TextKey` (`None` for the other classes, so it only breaks ties in text).
    entries
        .sort_by(|(a, ka, ..), (b, kb, ..)| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then_with(|| ka.cmp(kb)).then_with(|| (&a.2, &a.3).cmp(&(&b.2, &b.3))));
    entries.into_iter().map(|(_, _, t, on)| (t, on)).collect()
}
