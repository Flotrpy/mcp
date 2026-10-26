//! End-to-end tests of `handset-mcp` with a fake `adb` on PATH.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use mcp_kit::testing::stateless_meta;
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_handset-mcp");
const FIX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

struct Run {
    log: String,
}

impl Run {
    fn new(name: &str) -> Self {
        let log = std::env::temp_dir()
            .join(format!("handset-{}-{name}.log", std::process::id()))
            .display()
            .to_string();
        let _ = std::fs::remove_file(&log);
        Self { log }
    }

    /// Send tool calls to one server process (so refs persist between calls).
    fn calls(&self, calls: &[(&str, Value)], next_screen: bool) -> Vec<Value> {
        let path = format!("{FIX}/bin:{}", std::env::var("PATH").unwrap_or_default());
        let mut cmd = Command::new(BIN);
        cmd.env("PATH", path)
            .env("HANDSET_LOG", &self.log)
            .env("HANDSET_SCREEN", format!("{FIX}/login.xml"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if next_screen {
            cmd.env("HANDSET_NEXT_SCREEN", format!("{FIX}/inbox.xml"));
        }
        let mut child = cmd.spawn().unwrap();
        {
            let mut stdin = child.stdin.take().unwrap();
            for (i, (name, args)) in calls.iter().enumerate() {
                let req = json!({"jsonrpc": "2.0", "id": i + 1, "method": "tools/call",
                    "params": {"name": name, "arguments": args, "_meta": stateless_meta()}});
                writeln!(stdin, "{req}").unwrap();
                // Calls are handled concurrently; pace them so each sees the last.
                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        }
        let out = child.wait_with_output().unwrap();
        let mut res: Vec<Value> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v.get("id").is_some())
            .collect();
        res.sort_by_key(|v| v["id"].as_i64());
        res.into_iter().map(|v| v["result"].clone()).collect()
    }

    fn adb_calls(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
}

fn text(r: &Value) -> &str {
    r["content"][0]["text"].as_str().unwrap_or("")
}

#[test]
fn lists_the_fake_emulator() {
    let run = Run::new("list");
    let r = run.calls(&[("list_devices", json!({}))], false);
    assert!(text(&r[0]).contains("emulator-5554"), "{}", r[0]);
    assert_eq!(r[0]["structuredContent"]["devices"][0]["name"], "Pixel 8");
}

#[test]
fn snapshot_collapses_the_wrapper_and_labels_the_button() {
    let run = Run::new("snap");
    let r = run.calls(&[("snapshot", json!({}))], false);
    assert_eq!(text(&r[0]), "e1 text \"Welcome back\"\ne2 input \"Email\"\ne3 button \"Sign in\"");
}

#[test]
fn tap_by_ref_navigates_and_returns_the_new_screen() {
    let run = Run::new("nav");
    let r = run.calls(&[("snapshot", json!({})), ("act", json!({"action": "tap", "ref": "e3"}))], true);
    let t = text(&r[1]);
    assert!(t.starts_with("Tapped e3 \"Sign in\""), "{t}");
    assert!(t.contains("+ e4 text \"Inbox\""), "{t}");
    assert!(t.contains("- e3 button \"Sign in\""), "{t}");
    assert!(t.contains("New screen:"), "{t}");
    assert!(run.adb_calls().contains("adb -s emulator-5554 shell input tap 540 1070"));
}

#[test]
fn type_into_a_field_by_label() {
    let run = Run::new("type");
    let r = run.calls(
        &[(
            "act",
            json!({"action": "type", "target_text": "Email", "text": "jo@example.com", "submit": true}),
        )],
        false,
    );
    assert!(text(&r[0]).contains("Typed 14 characters"), "{}", r[0]);
    let calls = run.adb_calls();
    assert!(calls.contains("shell input text jo@example.com"), "{calls}");
    assert!(calls.contains("KEYCODE_ENTER"));
}

#[test]
fn bad_arguments_and_stale_refs_are_recoverable_errors() {
    let run = Run::new("errors");
    let r = run
        .calls(&[("act", json!({"action": "tap"})), ("act", json!({"action": "tap", "ref": "e42"}))], false);
    assert_eq!(r[0]["isError"], true);
    assert!(text(&r[0]).contains("needs `ref`"));
    assert_eq!(r[1]["isError"], true);
    assert!(text(&r[1]).contains("call snapshot"));
}

#[test]
fn wait_for_finds_visible_text() {
    let run = Run::new("wait");
    let r = run.calls(&[("wait_for", json!({"target": "Welcome", "timeout_seconds": 2}))], false);
    assert!(text(&r[0]).contains("appeared"), "{}", r[0]);
}
