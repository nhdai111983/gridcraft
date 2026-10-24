//! NexusOffice: what the engine writes into the user's files names Nexus Sheet, not upstream's
//! product (spec §4.4: default author, `<Application>`, PDF Producer/Creator, `INFO`). File-format
//! keys (`gridcraft://`, `GRIDCRAFT_*`, the MCP server's machine name) are deliberately unchanged.

use std::io::Read;

use gridcraft_engine::Session;
use serde_json::json;

/// A fresh directory for one test (tests run in parallel), removed by the test when it ends.
fn tmp_dir(test: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus-sheet-brand-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn part(xlsx: &std::path::Path, name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(xlsx).unwrap()).unwrap();
    let mut out = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut out).unwrap();
    out
}

fn all_parts(xlsx: &std::path::Path) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(xlsx).unwrap()).unwrap();
    let mut all = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            all.push_str(&s);
        }
    }
    all
}

#[test]
fn default_author_is_nexus() {
    let mut s = Session::new();
    s.new_workbook();
    assert_eq!(s.prefs.user_name, "Nexus Sheet User");
    s.execute("cell.set", json!({"cell": "A1", "input": "Xin chào"})).unwrap();
    // A note added without an author is signed with the default user name.
    s.execute("review.newNote", json!({"cell": "A1", "text": "ghi chú"})).unwrap();
    let dir = tmp_dir("author");
    let path = dir.join("new.xlsx");
    s.execute("file.saveAs", json!({"path": path.to_str().unwrap()})).unwrap();

    let app = part(&path, "docProps/app.xml");
    assert!(app.contains("<Application>Nexus Sheet</Application>"), "{app}");
    let all = all_parts(&path);
    assert!(all.contains("Nexus Sheet User"), "the note's author is the Nexus default");
    assert!(!all.contains("GridCraft"), "no upstream product name in a saved workbook");

    // Reopened, the note keeps its author.
    let mut reopened = Session::new();
    reopened.execute("file.open", json!({"path": path.to_str().unwrap()})).unwrap();
    let doc = reopened.doc().unwrap();
    let authors: Vec<&str> = doc.wb.active().unwrap().comments.values().map(|c| c.author.as_str()).collect();
    assert_eq!(authors, ["Nexus Sheet User"]);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn pdf_export_names_nexus_sheets() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    let dir = tmp_dir("pdf");
    let path = dir.join("sheet.pdf");
    s.execute("file.saveAs", json!({"path": path.to_str().unwrap()})).unwrap();
    let pdf = std::fs::read(&path).unwrap();
    let has = |needle: &[u8]| pdf.windows(needle.len()).any(|w| w == needle);
    assert!(has(b"/Producer (Nexus Sheet PDF)"));
    assert!(has(b"/Creator (Nexus Sheet)"));
    assert!(!has(b"GridCraft"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn info_osversion_names_nexus_sheets() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "=INFO(\"osversion\")"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["value"], json!("Nexus Sheet"));
}

/// NexusOffice: new sheets are named in the interface's language when it sets a prefix
/// (Vietnamese: Trang tính1, Trang tính2); upstream's Sheet1 otherwise.
#[test]
fn sheets_take_the_interface_languages_name() {
    let mut s = gridcraft_engine::Session::new();
    s.new_workbook();
    assert_eq!(s.active().unwrap().wb.sheets[0].name, "Sheet1");
    s.sheet_name_prefix = "Trang tính".into();
    s.prefs.sheets_in_new_workbook = 2;
    s.new_workbook();
    let names: Vec<String> = s.active().unwrap().wb.sheets.iter().map(|sh| sh.name.clone()).collect();
    assert_eq!(names, ["Trang tính1", "Trang tính2"]);
    s.execute("home.insertSheet", serde_json::json!({})).unwrap();
    assert!(s.active().unwrap().wb.sheets.iter().any(|sh| sh.name == "Trang tính3"));
    s.sheet_name_prefix = "  ".into();
    assert_eq!(s.active().unwrap().wb.next_sheet_name_with(&s.sheet_name_prefix), "Sheet1", "a blank prefix is Sheet");
}
