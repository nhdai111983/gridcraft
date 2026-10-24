//! NexusOffice: Open and Save As dialogs a host shows without freezing its window.
//!
//! A host with [`Services::file_dialog`](crate::Services::file_dialog) (the NexusOffice window,
//! which shows the system dialog off the UI thread) answers later through a channel;
//! [`SheetApp::logic`](crate::SheetApp::logic) picks the answer up on a later frame and finishes
//! what the dialog was for (open the workbook, save it and maybe close it, export a copy). A host
//! with only the blocking `pick_open` / `pick_save` (the stand-alone app) keeps upstream's flow.
//! One dialog at a time: asking for another while one is open does nothing.

use std::sync::mpsc::{Receiver, TryRecvError};

use serde_json::{Value as Json, json};

use crate::SheetApp;

/// A file dialog the app asks the host to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogRequest {
    /// Pick a file to open; `purpose` says what for (`workbook`).
    Open { purpose: String },
    /// Pick a path to save to, given a suggested file name (its extension names the format:
    /// `xlsx`, or `pdf`, `csv`, `html` for an export).
    Save { name: String },
}

/// The host's answer: the picked path, `Ok(None)` when the user cancelled, or an error when no
/// system dialog could be shown.
pub type FileDialogAnswer = Result<Option<String>, String>;

/// The file types Open offers, as (label, extensions).
pub const OPEN_FILTERS: &[(&str, &[&str])] = &[
    ("Spreadsheets", &["xlsx", "xlsm", "xls", "xlsb", "ods", "csv", "tsv", "txt", "json"]),
    ("Excel Workbook", &["xlsx", "xlsm"]),
    ("CSV", &["csv"]),
];

/// The formats Save As writes, as (label, extension); the suggested name's own comes first.
pub fn save_types(name: &str) -> Vec<(&'static str, &'static str)> {
    const TYPES: [(&str, &str); 4] =
        [("Excel Workbook (.xlsx)", "xlsx"), ("CSV UTF-8 (.csv)", "csv"), ("Web Page (.html)", "html"), ("PDF (.pdf)", "pdf")];
    let ext = std::path::Path::new(name).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    let first = TYPES.iter().find(|(_, x)| ext.as_deref() == Some(*x)).copied().unwrap_or(TYPES[0]);
    std::iter::once(first).chain(TYPES.iter().copied().filter(|t| *t != first)).collect()
}

/// What to do with the path picked in a dialog.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AfterPick {
    /// Open the picked workbook.
    OpenWorkbook,
    /// Save the workbook there; then close it when "Save changes?" asked for the save.
    Save { then_close: bool },
    /// Export a PDF there (other `file.exportPdf` params kept).
    ExportPdf(Json),
    /// Save a copy there with this command (`file.saveAs`, `file.exportCsv`, `file.exportHtml`,
    /// `file.exportPdf`), in the format `ext` names.
    SaveCopy { command: String, ext: String },
}

/// A dialog the host is showing; the answer arrives on `rx`.
pub(crate) struct PendingDialog {
    rx: Receiver<FileDialogAnswer>,
    pub(crate) after: AfterPick,
}

impl SheetApp {
    /// Whether a host file dialog is open (its answer hasn't arrived yet).
    pub fn file_dialog_open(&self) -> bool {
        self.file_dialog.is_some()
    }

    /// Ask the host's non-blocking dialog and do `after` with the answer on a later frame. `false`
    /// when the host has none (the caller uses the blocking picker). Asking while another dialog
    /// is open does nothing (and counts as asked).
    pub(crate) fn ask_file(&mut self, req: FileDialogRequest, after: AfterPick) -> bool {
        if self.file_dialog.is_some() {
            return true;
        }
        let Some(ask) = &self.services.file_dialog else { return false };
        let rx = ask(req);
        self.file_dialog = Some(PendingDialog { rx, after });
        true
    }

    /// "Save changes?" answered Save on a workbook with no file: close it once Save As has saved.
    pub(crate) fn close_after_pending_save(&mut self) {
        if let Some(p) = &mut self.file_dialog
            && matches!(p.after, AfterPick::Save { .. })
        {
            p.after = AfterPick::Save { then_close: true };
        }
    }

    /// Picks up the host's answer, if it has come. Called every frame from `logic`.
    pub(crate) fn poll_file_dialog(&mut self) {
        let answer = match &self.file_dialog {
            None => return,
            Some(p) => match p.rx.try_recv() {
                Err(TryRecvError::Empty) => return,
                // The host went away without answering: as cancelled.
                Err(TryRecvError::Disconnected) => Ok(None),
                Ok(answer) => answer,
            },
        };
        let Some(pending) = self.file_dialog.take() else { return };
        match answer {
            Ok(Some(path)) if !path.is_empty() => self.finish_pick(path, pending.after),
            Ok(_) => {}
            Err(why) => {
                self.message = Some((crate::nexus::APP_NAME.into(), format!("{}: {why}", self.ui.language.tr("No file dialog could be shown"))));
            }
        }
    }

    fn finish_pick(&mut self, path: String, after: AfterPick) {
        match after {
            AfterPick::OpenWorkbook => self.open_path(&path),
            AfterPick::Save { then_close } => {
                let path = if std::path::Path::new(&path).extension().is_none() { format!("{path}.xlsx") } else { path };
                self.run_or_alert("file.save", json!({"path": path}));
                if then_close && self.session.active().is_some_and(|d| !d.is_dirty()) {
                    self.run_or_alert("file.close", json!({"force": true}));
                }
            }
            AfterPick::ExportPdf(mut params) => {
                let path = std::path::Path::new(&path).with_extension("pdf").to_string_lossy().into_owned();
                match &mut params {
                    Json::Object(map) => {
                        map.insert("path".into(), Json::String(path));
                    }
                    other => *other = json!({"path": path}),
                }
                self.run_or_alert("file.exportPdf", params);
            }
            AfterPick::SaveCopy { command, ext } => {
                // The dialog may keep another type's extension; the copy is written as `ext`.
                let path = std::path::Path::new(&path).with_extension(&ext).to_string_lossy().into_owned();
                self.run_or_alert(&command, json!({"path": path}));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_types_put_the_suggested_format_first() {
        assert_eq!(save_types("Bảng tính 1.xlsx").first().map(|t| t.1), Some("xlsx"));
        assert_eq!(save_types("Báo cáo.PDF").first().map(|t| t.1), Some("pdf"));
        assert_eq!(save_types("không đuôi").first().map(|t| t.1), Some("xlsx"));
        assert_eq!(save_types("lạ.xyz").len(), 4);
        for name in ["a.csv", "b.html", "c"] {
            let t = save_types(name);
            assert_eq!(t.len(), 4, "{name}: every format once");
        }
    }
}
