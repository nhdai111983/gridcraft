//! NexusOffice (QC-SHT-06): Insert Function's syntax and description in Vietnamese for the
//! functions people use most. Function names stay English, as in Excel's Vietnamese edition;
//! the argument names and the description are Vietnamese. Other functions keep the English help.

/// `(name, syntax, description)`, with the Vietnamese list separator `;`.
const HELP: &[(&str, &str, &str)] = &[
    ("ABS", "ABS(số)", "Trả về giá trị tuyệt đối của một số."),
    ("SUM", "SUM(số1; [số2]; ...)", "Cộng tất cả các số trong các đối số."),
    ("AVERAGE", "AVERAGE(số1; [số2]; ...)", "Trả về trung bình cộng của các đối số."),
    ("MIN", "MIN(số1; [số2]; ...)", "Trả về giá trị nhỏ nhất trong các đối số."),
    ("MAX", "MAX(số1; [số2]; ...)", "Trả về giá trị lớn nhất trong các đối số."),
    ("COUNT", "COUNT(giá_trị1; [giá_trị2]; ...)", "Đếm số ô chứa số."),
    ("COUNTA", "COUNTA(giá_trị1; [giá_trị2]; ...)", "Đếm số ô không trống."),
    ("COUNTBLANK", "COUNTBLANK(vùng)", "Đếm số ô trống trong một vùng."),
    ("IF", "IF(điều_kiện; [giá_trị_nếu_đúng]; [giá_trị_nếu_sai])", "Trả về một giá trị khi điều kiện đúng và giá trị khác khi điều kiện sai."),
    ("IFS", "IFS(điều_kiện1; giá_trị1; [điều_kiện2; giá_trị2]; ...)", "Trả về giá trị ứng với điều kiện đúng đầu tiên."),
    ("IFERROR", "IFERROR(giá_trị; giá_trị_nếu_lỗi)", "Trả về giá_trị_nếu_lỗi khi biểu thức bị lỗi, nếu không trả về chính biểu thức."),
    ("AND", "AND(lô_gic1; [lô_gic2]; ...)", "Trả về TRUE khi mọi đối số đều TRUE."),
    ("OR", "OR(lô_gic1; [lô_gic2]; ...)", "Trả về TRUE khi có ít nhất một đối số TRUE."),
    ("NOT", "NOT(lô_gic)", "Đảo giá trị lô-gic của đối số."),
    ("TRUE", "TRUE()", "Trả về giá trị lô-gic TRUE."),
    ("FALSE", "FALSE()", "Trả về giá trị lô-gic FALSE."),
    ("SUMIF", "SUMIF(vùng; tiêu_chí; [vùng_tính_tổng])", "Cộng các ô thỏa một tiêu chí."),
    ("SUMIFS", "SUMIFS(vùng_tính_tổng; vùng_tiêu_chí1; tiêu_chí1; ...)", "Cộng các ô thỏa nhiều tiêu chí."),
    ("COUNTIF", "COUNTIF(vùng; tiêu_chí)", "Đếm số ô thỏa một tiêu chí."),
    ("COUNTIFS", "COUNTIFS(vùng_tiêu_chí1; tiêu_chí1; ...)", "Đếm số ô thỏa nhiều tiêu chí."),
    ("AVERAGEIF", "AVERAGEIF(vùng; tiêu_chí; [vùng_tính_trung_bình])", "Trả về trung bình cộng của các ô thỏa một tiêu chí."),
    (
        "VLOOKUP",
        "VLOOKUP(giá_trị_tìm; bảng; cột_trả_về; [tìm_gần_đúng])",
        "Tìm một giá trị ở cột đầu của bảng và trả về giá trị cùng hàng ở cột chỉ định.",
    ),
    (
        "HLOOKUP",
        "HLOOKUP(giá_trị_tìm; bảng; hàng_trả_về; [tìm_gần_đúng])",
        "Tìm một giá trị ở hàng đầu của bảng và trả về giá trị cùng cột ở hàng chỉ định.",
    ),
    (
        "XLOOKUP",
        "XLOOKUP(giá_trị_tìm; mảng_tìm; mảng_trả_về; [nếu_không_thấy]; [kiểu_khớp]; [kiểu_tìm])",
        "Tìm một giá trị trong một vùng và trả về mục tương ứng ở vùng khác.",
    ),
    ("INDEX", "INDEX(mảng; số_hàng; [số_cột])", "Trả về giá trị ở hàng và cột chỉ định của một vùng."),
    ("MATCH", "MATCH(giá_trị_tìm; mảng_tìm; [kiểu_khớp])", "Trả về vị trí của một giá trị trong một vùng."),
    ("ROUND", "ROUND(số; số_chữ_số)", "Làm tròn một số tới số chữ số chỉ định."),
    ("ROUNDUP", "ROUNDUP(số; số_chữ_số)", "Làm tròn lên, ra xa số 0."),
    ("ROUNDDOWN", "ROUNDDOWN(số; số_chữ_số)", "Làm tròn xuống, về phía số 0."),
    ("INT", "INT(số)", "Làm tròn xuống tới số nguyên gần nhất."),
    ("MOD", "MOD(số; số_chia)", "Trả về số dư của phép chia."),
    ("TODAY", "TODAY()", "Trả về ngày hôm nay."),
    ("NOW", "NOW()", "Trả về ngày và giờ hiện tại."),
    ("DATE", "DATE(năm; tháng; ngày)", "Trả về ngày ứng với năm, tháng, ngày."),
    ("YEAR", "YEAR(ngày)", "Trả về năm của một ngày."),
    ("MONTH", "MONTH(ngày)", "Trả về tháng của một ngày (1 đến 12)."),
    ("DAY", "DAY(ngày)", "Trả về ngày trong tháng (1 đến 31)."),
    ("LEFT", "LEFT(văn_bản; [số_ký_tự])", "Trả về các ký tự đầu của chuỗi."),
    ("RIGHT", "RIGHT(văn_bản; [số_ký_tự])", "Trả về các ký tự cuối của chuỗi."),
    ("MID", "MID(văn_bản; vị_trí_bắt_đầu; số_ký_tự)", "Trả về các ký tự ở giữa chuỗi, từ vị trí chỉ định."),
    ("LEN", "LEN(văn_bản)", "Trả về số ký tự của chuỗi."),
    ("TRIM", "TRIM(văn_bản)", "Bỏ khoảng trắng thừa trong chuỗi."),
    ("UPPER", "UPPER(văn_bản)", "Chuyển chuỗi thành chữ hoa."),
    ("LOWER", "LOWER(văn_bản)", "Chuyển chuỗi thành chữ thường."),
    ("CONCAT", "CONCAT(văn_bản1; [văn_bản2]; ...)", "Nối các chuỗi lại với nhau."),
    ("TEXT", "TEXT(giá_trị; mã_định_dạng)", "Định dạng một số thành chuỗi theo mã định dạng."),
    ("VALUE", "VALUE(văn_bản)", "Chuyển chuỗi biểu diễn số thành số."),
];

/// The Vietnamese syntax of `name` (canonical, upper-case), if there is one.
pub(crate) fn signature(name: &str) -> Option<&'static str> {
    HELP.iter().find(|(n, ..)| *n == name).map(|(_, s, _)| *s)
}

/// The Vietnamese description of `name` (canonical, upper-case), if there is one.
pub(crate) fn description(name: &str) -> Option<&'static str> {
    HELP.iter().find(|(n, ..)| *n == name).map(|(.., d)| *d)
}
