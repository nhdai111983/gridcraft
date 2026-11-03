//! NexusOffice: what typing a value into the grid does beyond storing it, as in Excel.
//!
//! QC-SHT-12: a formula typed into a General cell takes the number format of the first cell it
//! reads (`=SUM(B2:B5)` over `1.234,50` shows `2.001.979,25`, not `2001979`), for formulas
//! that keep their inputs' unit: SUM-like functions, rounding, and plain arithmetic.
//!
//! QC-SHT-10: typing a date or a formatted number into a column of default width
//! widens the column to show it, as Excel does, instead of leaving `#######`. A column whose width
//! someone set is left alone (Excel shows `###` there too), and so are General numbers (they
//! narrow to fewer digits instead) and formulas.

use gridcraft_core::{CellRef, Value};
use gridcraft_formula::{BinOp, Expr, SheetSel};
use gridcraft_model::Workbook;

/// Functions whose result is in the unit of their first argument.
const KEEPS_UNIT: &[&str] = &[
    "SUM",
    "AVERAGE",
    "MIN",
    "MAX",
    "MEDIAN",
    "SUBTOTAL",
    "AGGREGATE",
    "ROUND",
    "ROUNDUP",
    "ROUNDDOWN",
    "ABS",
    "SUMIF",
    "SUMIFS",
    "AVERAGEIF",
    "AVERAGEIFS",
    "MINIFS",
    "MAXIFS",
];

/// Whether `e` gives a result in its inputs' unit (see [`KEEPS_UNIT`]).
fn keeps_unit(e: &Expr) -> bool {
    match e {
        Expr::Paren(x) | Expr::Unary(_, x) => keeps_unit(x),
        Expr::Ref(_) => true,
        Expr::Call(name, _) => KEEPS_UNIT.contains(&name.as_str()),
        Expr::Binary(op, a, b) => matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div) && (keeps_unit(a) || keeps_unit(b)),
        _ => false,
    }
}

/// Gives the formula just typed at `at` the number format of the first cell it reads, when the
/// cell is General and the formula keeps its inputs' unit. Returns whether it did.
pub(crate) fn inherit_number_format(wb: &mut Workbook, sheet: usize, at: CellRef) -> bool {
    let Some(sh) = wb.sheet(sheet) else { return false };
    let Some(cell) = sh.cell(at) else { return false };
    let Some(expr) = cell.formula.as_ref().and_then(|f| f.expr.clone()) else { return false };
    let style = cell.style;
    if !wb.styles.get(style).num_fmt.as_str().eq_ignore_ascii_case("General") || !keeps_unit(&expr) {
        return false;
    }
    let Some(first) = expr.references().into_iter().find(|r| r.sheet == SheetSel::Current).map(|r| r.range().start) else { return false };
    let code = wb.styles.get(sh.style_id(first)).num_fmt.as_str().to_string();
    if code.eq_ignore_ascii_case("General") || code == "@" {
        return false;
    }
    let derived = wb.styles.derive(style, |s| s.num_fmt = gridcraft_model::NumFmt::new(&code));
    let Some(sh) = wb.sheet_mut(sheet) else { return false };
    let Some(c) = sh.cells.get(at).cloned() else { return false };
    sh.set_cell(at, gridcraft_model::Cell { style: derived, ..c });
    true
}

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

/// The word new table headers are named with (`Column1`; `Cột1` in a Vietnamese region, as
/// Excel names them in the system's language).
pub(crate) fn column_word(locale: gridcraft_core::Locale) -> &'static str {
    match locale {
        gridcraft_core::Locale::ViVn => "Cột",
        _ => "Column",
    }
}
