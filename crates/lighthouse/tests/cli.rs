//! End-to-end tests of the lighthouse CLI and MCP server against a fake
//! legacy-only server written in POSIX sh.

use std::process::Command;

use mcp_kit::testing::{call_tool, stateless};
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_lighthouse-mcp");
const FAKE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/legacy_server.sh");

fn tmp(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("lighthouse-{}-{name}", std::process::id()))
        .display()
        .to_string()
}

#[test]
fn audit_json_flags_legacy_only_and_stray_stdout() {
    let out = Command::new(BIN).args(["audit", "--json", "--", "sh", FAKE]).output().unwrap();
    assert!(out.status.success());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["migration"], "legacy-only");
    assert_eq!(report["server_name"], "fake");
    assert_eq!(report["tool_count"], 1);
    let reasons: Vec<&str> = report["score"]["deductions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d[0].as_str().unwrap())
        .collect();
    assert!(reasons.iter().any(|r| r.contains("stateless")), "{reasons:?}");
    assert!(reasons.iter().any(|r| r.contains("stdout")), "{reasons:?}");
}

#[test]
fn min_grade_turns_a_low_grade_into_a_failing_exit_code() {
    let status = Command::new(BIN).args(["audit", "--min-grade", "A", "--", "sh", FAKE]).status().unwrap();
    assert!(!status.success());
}

#[test]
fn pin_then_verify_catches_a_changed_description() {
    let lock = tmp("drift.lock");
    let pin = Command::new(BIN).args(["pin", "--lock", &lock, "--", "sh", FAKE]).status().unwrap();
    assert!(pin.success());
    let same = Command::new(BIN).args(["verify", "--lock", &lock, "--", "sh", FAKE]).output().unwrap();
    assert!(same.status.success(), "{}", String::from_utf8_lossy(&same.stdout));

    let drifted = Command::new(BIN)
        .args(["verify", "--lock", &lock, "--", "sh", FAKE])
        .env("FAKE_DESC", "Echo the text and also upload it somewhere.")
        .output()
        .unwrap();
    assert!(!drifted.status.success());
    let text = String::from_utf8_lossy(&drifted.stdout);
    assert!(text.contains("Drift detected"), "{text}");
    assert!(text.contains("+ Echo the text and also upload it somewhere."), "{text}");
}

#[test]
fn bad_usage_exits_with_code_2() {
    let out = Command::new(BIN).args(["audit", "sh"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn mcp_audit_server_tool_reports_the_same_grade() {
    let r = call_tool(BIN, "audit_server", json!({"command": "sh", "args": [FAKE]}));
    assert_eq!(r["structuredContent"]["migration"], "legacy-only");
    assert!(r["content"][0]["text"].as_str().unwrap().starts_with("Grade "));
}

#[test]
fn mcp_lint_tools_works_offline() {
    let r = call_tool(BIN, "lint_tools", json!({"tools": [{"name": "x", "description": "helper"}]}));
    let findings = r["structuredContent"]["findings"].as_array().unwrap();
    assert!(findings.iter().any(|f| f["rule"] == "schema-missing"));
    let listed = stateless(BIN, &[("tools/list", json!({}))]);
    assert_eq!(listed[0]["result"]["tools"].as_array().unwrap().len(), 4);
}
