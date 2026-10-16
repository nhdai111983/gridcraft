//! NexusOffices: what the engine writes into the user's files names Nexus Sheets, not upstream's
//! product (spec §4.4: default author, `<Application>`, PDF Producer/Creator, `INFO`). File-format
//! keys (`gridcraft://`, `GRIDCRAFT_*`, the MCP server's machine name) are deliberately unchanged.

use std::io::Read;

use gridcraft_engine::Session;
use serde_json::json;

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gridcraft-nexus-brand-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
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
    assert_eq!(s.prefs.user_name, "Nexus Sheets User");
    s.execute("cell.set", json!({"cell": "A1", "input": "Xin chào"})).unwrap();
    // A note added without an author is signed with the default user name.
    s.execute("review.newNote", json!({"cell": "A1", "text": "ghi chú"})).unwrap();
    let path = tmp("new.xlsx");
    s.execute("file.saveAs", json!({"path": path.to_str().unwrap()})).unwrap();

    let app = part(&path, "docProps/app.xml");
    assert!(app.contains("<Application>Nexus Sheets</Application>"), "{app}");
    let all = all_parts(&path);
    assert!(all.contains("Nexus Sheets User"), "the note's author is the Nexus default");
    assert!(!all.contains("GridCraft"), "no upstream product name in a saved workbook");

    // Reopened, the note keeps its author.
    let mut reopened = Session::new();
    reopened.execute("file.open", json!({"path": path.to_str().unwrap()})).unwrap();
    let doc = reopened.doc().unwrap();
    let authors: Vec<&str> = doc.wb.active().unwrap().comments.values().map(|c| c.author.as_str()).collect();
    assert_eq!(authors, ["Nexus Sheets User"]);
}

#[test]
fn pdf_export_names_nexus_sheets() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    let path = tmp("sheet.pdf");
    s.execute("file.saveAs", json!({"path": path.to_str().unwrap()})).unwrap();
    let pdf = std::fs::read(&path).unwrap();
    let has = |needle: &[u8]| pdf.windows(needle.len()).any(|w| w == needle);
    assert!(has(b"/Producer (Nexus Sheets PDF)"));
    assert!(has(b"/Creator (Nexus Sheets)"));
    assert!(!has(b"GridCraft"));
}

#[test]
fn info_osversion_names_nexus_sheets() {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "=INFO(\"osversion\")"})).unwrap();
    assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["value"], json!("Nexus Sheets"));
}
