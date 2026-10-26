//! NexusOffice (QC-SHT-10): typing a date or a formatted number into a column of default width
//! widens the column to show it, as Excel does, instead of leaving `#######`. A column whose width
//! someone set is left alone (Excel shows `###` there too), and so are General numbers (they
//! narrow to fewer digits instead) and formulas.

use gridcraft_core::{CellRef, Value};
use gridcraft_model::Workbook;

/// Widest column this ever makes (Excel's 255 characters).
const MAX_WIDTH: f32 = 1785.0;

/// Widens column `at.col` of `sheet` in `wb` when the number just typed at `at` doesn't fit.
/// Returns whether the width changed.
pub(crate) fn widen_for_typed_number(wb: &mut Workbook, sheet: usize, at: CellRef) -> bool {
    let Some(sh) = wb.sheet(sheet) else { return false };
    let Some(cell) = sh.cell(at) else { return false };
    if cell.formula.is_some() || !matches!(cell.value, Value::Number(_)) {
        return false;
    }
    // A width set by hand (or read from the file) stays.
    if sh.cols.get(&at.col).is_some_and(|c| c.size.is_some() || c.hidden) || sh.merge_at(at).is_some_and(|m| !m.is_single()) {
        return false;
    }
    let style = wb.styles.get(sh.style_id(at));
    if style.num_fmt.as_str().eq_ignore_ascii_case("General") {
        return false;
    }
    let text = crate::display::value_text(wb, sh, at);
    let need = crate::cmd::format::approx_text_width(&text, style.font.size, style.font.bold) + 6.0 + style.align.indent as f32 * 9.0;
    if !need.is_finite() || need <= sh.col_width(at.col) {
        return false;
    }
    let Some(sh) = wb.sheet_mut(sheet) else { return false };
    let e = sh.cols.entry(at.col).or_default();
    e.size = Some(need.clamp(8.0, MAX_WIDTH).ceil());
    true
}
