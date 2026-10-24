//! NexusOffice: Vietnamese formula entry keeps English function names (as Excel's Vietnamese
//! version does) with the vi-VN separators: decimal `,`, arguments `;`, array columns `\`.
use gridcraft_formula::{FormulaLocale, parse, parse_input, print, print_input};

fn input(src: &str) -> String {
    print(&parse_input(src, FormulaLocale::Vi, |_| false).unwrap())
}

fn shown(canonical: &str) -> String {
    print_input(&parse(canonical).unwrap(), FormulaLocale::Vi, |_| false).unwrap()
}

#[test]
fn vietnamese_separators_with_english_names() {
    for (src, expected) in [
        ("=SUM(1,5;2,25)", "SUM(1.5,2.25)"),
        ("IF(A1>0;\"Có\";\"Không\")", "IF(A1>0,\"Có\",\"Không\")"),
        ("ROUND(A1*1,1;0)", "ROUND(A1*1.1,0)"),
        ("SUM({1\\2;3\\4};5)", "SUM({1,2;3,4},5)"),
        ("IF(TRUE;FALSE;)", "IF(TRUE,FALSE,)"),
        ("TEXT(A1;\"#.##0,00\")", "TEXT(A1,\"#.##0,00\")"),
        ("'Trang tính1'!A1+1", "'Trang tính1'!A1+1"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
}

#[test]
fn spanish_names_are_not_vietnamese() {
    // SUMA is no alias in Vietnamese: it stays an unknown name, and VERDADERO is no boolean.
    assert_eq!(input("SUMA(1;2)"), "SUMA(1,2)");
    assert_eq!(input("VERDADERO"), "VERDADERO");
}

#[test]
fn canonical_formulas_show_in_vietnamese_syntax() {
    for (canonical, expected) in
        [("IF(TRUE,SUM(1.5,2.25),0)", "IF(TRUE;SUM(1,5;2,25);0)"), ("SUM({1,2;3,4})", "SUM({1\\2;3\\4})"), ("\"1,5\"&\"a;b\"", "\"1,5\"&\"a;b\"")]
    {
        assert_eq!(shown(canonical), expected, "{canonical}");
        assert_eq!(input(expected), canonical, "{expected} reads back");
    }
}
