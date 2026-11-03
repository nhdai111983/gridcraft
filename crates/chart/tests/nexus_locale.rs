//! NexusOffice: chart axis, data and category labels in the workbook's regional format
//! (`1.234,50`, `25/10/2026`), as Excel's follow the system's. en-US is unchanged.

use std::sync::Arc;

use gridcraft_chart::{ApproxMeasure, Prim, render, resolve};
use gridcraft_core::date::serial_from_ymd;
use gridcraft_core::{CellRef, DateSystem, Locale, Value};
use gridcraft_model::{Anchor, Chart, ChartKind, LegendPos, NumFmt, Series, Style, Workbook};

fn chart(kind: ChartKind) -> Chart {
    Chart {
        id: 1,
        kind,
        anchor: Anchor::default(),
        title: None,
        series: vec![Series {
            name: Some("Sheet1!$C$1".into()),
            categories: Some("Sheet1!$A$2:$A$4".into()),
            values: "Sheet1!$B$2:$B$4".into(),
            ..Series::default()
        }],
        legend: LegendPos::Bottom,
        data_labels: true,
        gridlines: true,
        style: 0,
        x_title: None,
        y_title: None,
        source: None,
        by_rows: false,
        pivot: None,
    }
}

/// Dates (short date) as categories, amounts with `#,##0.00` as values, a number as the name.
fn book(locale: Locale) -> Workbook {
    let mut wb = Workbook::new();
    wb.locale = locale;
    let date = wb.styles.intern(Style { num_fmt: NumFmt(Arc::from("m/d/yyyy")), ..Style::default() });
    let money = wb.styles.intern(Style { num_fmt: NumFmt(Arc::from("#,##0.00")), ..Style::default() });
    let s = wb.sheet_mut(0).unwrap();
    s.set_value(CellRef::new(0, 2), Value::Number(2.5));
    for (i, (d, v)) in [(24, 1234.5), (25, 2500.25), (26, 3100.0)].into_iter().enumerate() {
        let r = i as u32 + 1;
        s.set_value(CellRef::new(r, 0), Value::Number(serial_from_ymd(DateSystem::D1900, 2026, 10, d).unwrap()));
        s.set_style(CellRef::new(r, 0), date);
        s.set_value(CellRef::new(r, 1), Value::Number(v));
        s.set_style(CellRef::new(r, 1), money);
    }
    wb
}

fn texts(prims: &[Prim]) -> Vec<String> {
    prims
        .iter()
        .filter_map(|p| match p {
            Prim::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn labels_follow_the_workbooks_regional_format() {
    let wb = book(Locale::ViVn);
    let c = chart(ChartKind::ColumnClustered);
    let d = resolve(&wb, 0, &c);
    assert_eq!(d.locale, Locale::ViVn);
    assert_eq!(d.categories, vec!["24/10/2026", "25/10/2026", "26/10/2026"]);
    assert_eq!(d.series[0].name, "2,5");
    let t = texts(&render(&c, &d, 640.0, 400.0, &ApproxMeasure));
    for label in ["1.234,50", "2.500,25", "3.100,00"] {
        assert!(t.iter().any(|x| x == label), "data label {label} in {t:?}");
    }
    // Value-axis ticks use the series' format, spelled vi-VN (`1.000,00`, not `1,000.00`).
    assert!(t.iter().any(|x| x == "1.000,00"), "{t:?}");
    assert!(!t.iter().any(|x| x.contains("1,000")), "{t:?}");
    // A General axis with fractional steps (scatter X) shows decimal commas.
    let mut sc = chart(ChartKind::Scatter);
    sc.series[0].categories = Some("{0.5,1,1.5}".into());
    sc.series[0].values = "{0.25,0.5,0.75}".into();
    let d = resolve(&wb, 0, &sc);
    let t = texts(&render(&sc, &d, 640.0, 400.0, &ApproxMeasure));
    assert!(t.iter().any(|x| x.contains(',') && !x.contains('.')), "{t:?}");
    assert!(!t.iter().any(|x| x.starts_with("0.")), "{t:?}");
}

#[test]
fn en_us_labels_are_unchanged() {
    let wb = book(Locale::EnUs);
    let c = chart(ChartKind::ColumnClustered);
    let d = resolve(&wb, 0, &c);
    assert_eq!(d.categories, vec!["10/24/2026", "10/25/2026", "10/26/2026"]);
    assert_eq!(d.series[0].name, "2.5");
    let t = texts(&render(&c, &d, 640.0, 400.0, &ApproxMeasure));
    for label in ["1,234.50", "2,500.25", "3,100.00", "1,000.00"] {
        assert!(t.iter().any(|x| x == label), "{label} in {t:?}");
    }
}

#[test]
fn every_kind_renders_in_vietnamese_without_panicking() {
    let wb = book(Locale::ViVn);
    for kind in [
        ChartKind::ColumnClustered,
        ChartKind::ColumnStacked100,
        ChartKind::BarClustered,
        ChartKind::Line,
        ChartKind::Pie,
        ChartKind::Doughnut,
        ChartKind::Scatter,
        ChartKind::Bubble,
        ChartKind::Area,
        ChartKind::Radar,
        ChartKind::Histogram,
        ChartKind::Waterfall,
        ChartKind::Funnel,
        ChartKind::Treemap,
        ChartKind::BoxWhisker,
        ChartKind::Combo,
    ] {
        let c = chart(kind);
        let d = resolve(&wb, 0, &c);
        let prims = render(&c, &d, 500.0, 300.0, &ApproxMeasure);
        for p in &prims {
            if let Prim::Text { text, .. } = p {
                assert!(!text.contains("1,234.5"), "{kind:?}: {text}");
            }
        }
    }
}
