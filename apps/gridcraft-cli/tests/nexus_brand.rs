//! NexusOffices: the CLI is installed as `nexus-sheets-cli`, so its usage and version text use that
//! name (the Cargo binary is still `gridcraft-cli`; packaging renames it).

use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gridcraft-cli")).args(args).output().expect("run the CLI")
}

#[test]
fn usage_and_version_name_nexus_sheets_cli() {
    let version = String::from_utf8_lossy(&run(&["--version"]).stdout).trim().to_string();
    assert_eq!(version, format!("nexus-sheets-cli {}", env!("CARGO_PKG_VERSION")));

    let help = String::from_utf8_lossy(&run(&["--help"]).stdout).to_string();
    assert!(help.starts_with("usage:\n  nexus-sheets-cli info"), "{help}");
    assert!(!help.contains("gridcraft"), "{help}");

    // Errors are prefixed with the installed name too.
    let o = run(&["frobnicate"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.starts_with("nexus-sheets-cli: "), "{err}");
    assert!(!err.contains("gridcraft"), "{err}");
}
