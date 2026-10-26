//! End-to-end tests against the compiled `binscope-mcp` binary, using small
//! system executables as fixtures.

use mcp_kit::testing::{call_tool, stateless};
use serde_json::json;

const BIN: &str = env!("CARGO_BIN_EXE_binscope-mcp");

fn fixture() -> Option<&'static str> {
    ["/bin/true", "/usr/bin/true", "/bin/ls"]
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
}

#[test]
fn lists_four_read_only_tools() {
    let r = stateless(BIN, &[("tools/list", json!({}))]);
    let tools = r[0]["result"]["tools"].as_array().unwrap();
    let mut names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort();
    assert_eq!(names, ["diff", "hexdump", "inspect", "overview"]);
    assert!(tools.iter().all(|t| t["annotations"]["readOnlyHint"] == true));
}

#[test]
fn overview_of_a_system_binary() {
    let Some(path) = fixture() else { return };
    let r = call_tool(BIN, "overview", json!({"path": path}));
    let sc = &r["structuredContent"];
    assert!(sc["counts"]["sections"].as_u64().unwrap() > 0);
    assert!(sc["next_steps"].as_array().unwrap().len() >= 2);
}

#[test]
fn strings_respect_the_token_budget() {
    let Some(path) = fixture() else { return };
    let r = call_tool(BIN, "inspect", json!({"path": path, "view": "strings", "max_tokens": 60}));
    let sc = &r["structuredContent"];
    assert!(sc["tokens"].as_u64().unwrap() <= 60 || sc["items"].as_array().unwrap().len() == 1);
    assert!(r["content"][0]["text"].as_str().unwrap().contains("omitted"));
}

#[test]
fn diff_of_a_file_with_itself_is_empty() {
    let Some(path) = fixture() else { return };
    let r = call_tool(BIN, "diff", json!({"old_path": path, "new_path": path}));
    assert_eq!(r["structuredContent"]["size_delta"], 0);
    assert!(r["structuredContent"]["highlights"].as_array().unwrap().is_empty());
}

#[test]
fn hexdump_starts_with_the_magic_number() {
    let Some(path) = fixture() else { return };
    let r = call_tool(BIN, "hexdump", json!({"path": path, "offset": 0, "length": 16}));
    let dump = r["structuredContent"]["dump"].as_str().unwrap();
    assert!(dump.starts_with("00000000  7f 45 4c 46") || dump.contains("|MZ") || dump.contains("cf fa ed fe"));
}

#[test]
fn missing_and_non_binary_files_are_tool_errors() {
    let r = call_tool(BIN, "overview", json!({"path": "/no/such/file"}));
    assert_eq!(r["isError"], true);
    let text_file = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let r = call_tool(BIN, "overview", json!({"path": text_file}));
    assert_eq!(r["isError"], true);
    assert!(r["content"][0]["text"].as_str().unwrap().contains("not a recognised"));
}
