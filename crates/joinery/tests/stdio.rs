//! End-to-end tests against the compiled `joinery-mcp` binary.

use mcp_kit::testing::{call_tool, legacy, stateless};
use serde_json::json;

const BIN: &str = env!("CARGO_BIN_EXE_joinery-mcp");

#[test]
fn discover_reports_joinery_and_stateless_support() {
    let r = stateless(BIN, &[("server/discover", json!({}))]);
    let result = &r[0]["result"];
    assert!(result["supportedVersions"].as_array().unwrap().iter().any(|v| v == "2026-07-28"));
    assert_eq!(result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"], "joinery");
}

#[test]
fn lists_the_same_four_tools_on_both_protocol_generations() {
    let names = |v: &serde_json::Value| {
        let mut n: Vec<String> = v["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        n.sort();
        n
    };
    let new = stateless(BIN, &[("tools/list", json!({}))]);
    let old = legacy(BIN, &[("tools/list", json!({}))]);
    let expected = ["check_shelf_sag", "design_furniture", "list_species", "nest_parts"];
    assert_eq!(names(&new[0]), expected);
    assert_eq!(names(&old.last().unwrap()), expected);
}

#[test]
fn every_tool_is_annotated_read_only() {
    let r = stateless(BIN, &[("tools/list", json!({}))]);
    for tool in r[0]["result"]["tools"].as_array().unwrap() {
        assert_eq!(tool["annotations"]["readOnlyHint"], true, "{}", tool["name"]);
    }
}

#[test]
fn design_furniture_flags_a_sagging_mdf_bookcase() {
    let result = call_tool(
        BIN,
        "design_furniture",
        json!({"kind": "bookcase", "width": 36, "height": 72, "depth": 12,
               "stock_thickness": 0.75, "adjustable_shelves": 4, "units": "in", "species": "mdf"}),
    );
    let sc = &result["structuredContent"];
    assert_eq!(sc["cut_list"]["units"], "in");
    assert_eq!(sc["shelf_checks"][0]["report"]["passes"], false);
    assert!(result["content"][0]["text"].as_str().unwrap().contains("sag visibly"));
}

#[test]
fn nest_parts_returns_an_svg_by_default() {
    let result = call_tool(
        BIN,
        "nest_parts",
        json!({"parts": [{"name": "side", "length": 1800, "width": 300, "quantity": 2}]}),
    );
    let sc = &result["structuredContent"];
    assert_eq!(sc["sheets_used"], 1);
    assert!(sc["svg"].as_str().unwrap().starts_with("<svg"));
}

#[test]
fn unknown_species_is_a_recoverable_tool_error() {
    let result = call_tool(
        BIN,
        "check_shelf_sag",
        json!({"span": 800, "depth": 250, "thickness": 18, "species": "balsa", "load_kg": 10}),
    );
    assert_eq!(result["isError"], true);
    assert!(result["content"][0]["text"].as_str().unwrap().contains("red-oak"));
}
