//! NexusOffices: the MCP server introduces itself as Nexus Sheets (spec §4.4, `serverInfo`), while
//! its machine name stays `gridcraft` (an identifier client configurations rely on).

use gridcraft_mcp::{Headless, Server, tool_definitions};
use serde_json::{Value, json};

#[test]
fn server_info_and_instructions_name_nexus_sheets() {
    let mut s = Server::new(Box::new(Headless::new()));
    let line = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}).to_string();
    let reply: Value = serde_json::from_str(&s.handle_line(&line).unwrap()).unwrap();
    let r = &reply["result"];
    assert_eq!(r["serverInfo"]["title"], "Nexus Sheets");
    assert_eq!(r["serverInfo"]["name"], "gridcraft", "the machine name is an identifier, kept");
    let instructions = r["instructions"].as_str().unwrap();
    assert!(instructions.starts_with("Nexus Sheets is a spreadsheet app"), "{instructions}");
    for text in [instructions.to_string(), serde_json::to_string(&tool_definitions()).unwrap()] {
        assert!(!text.contains("GridCraft") && !text.contains("ArtCraft") && !text.contains("discord"), "{text}");
    }
}

/// Command names in tool descriptions and hints are the installed binaries' (`nexus-sheets`,
/// `nexus-sheets-cli`), not upstream's.
#[test]
fn tool_texts_name_the_nexus_binaries() {
    let defs = serde_json::to_string(&tool_definitions()).unwrap();
    assert!(!defs.contains("gridcraft-cli"), "{defs}");
    assert!(defs.contains("nexus-sheets-cli mcp --connect 7979"), "{defs}");

    // A UI-only tool in headless mode explains how to start the app.
    let mut s = Server::new(Box::new(Headless::new()));
    let line =
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "screenshot", "arguments": {"path": "x.png"}}}).to_string();
    let reply = s.handle_line(&line).unwrap();
    assert!(reply.contains("nexus-sheets --control 7979") && reply.contains("nexus-sheets-cli mcp --connect 7979"), "{reply}");
    assert!(!reply.contains("gridcraft"), "{reply}");
}
