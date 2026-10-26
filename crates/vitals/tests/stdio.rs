//! End-to-end tests: generate a realistic Apple Health export, import it
//! through the compiled `vitals-mcp` binary and query it.

use std::fmt::Write as _;

use mcp_kit::testing::stateless;
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_vitals-mcp");

/// 35 days ending 2026-10-20. Phone and watch both count steps in the same
/// hours (the classic double count); HRV dips and resting HR rises on the
/// last day after a short night.
fn export_xml() -> String {
    let mut xml = String::from("<?xml version=\"1.0\"?>\n<HealthData>\n");
    let rec = |xml: &mut String, kind: &str, src: &str, unit: &str, start: &str, end: &str, value: &str| {
        let _ = writeln!(xml, r#" <Record type="{kind}" sourceName="{src}" unit="{unit}" startDate="{start}" endDate="{end}" value="{value}"/>"#);
    };
    for i in 0..35u32 {
        let day = 16 + i; // day-of-year offset from 2026-09-16
        let (m, d) = if day <= 30 { (9, day) } else { (10, day - 30) };
        let date = format!("2026-{m:02}-{d:02}");
        let last = i == 34;
        for h in [9, 13, 18] {
            let start = format!("{date} {h:02}:00:00 +0000");
            let end = format!("{date} {h:02}:30:00 +0000");
            rec(&mut xml, "HKQuantityTypeIdentifierStepCount", "Jo's iPhone", "count", &start, &end, "2000");
            rec(&mut xml, "HKQuantityTypeIdentifierStepCount", "Jo's Apple Watch", "count", &start, &end, "1900");
        }
        let morning = format!("{date} 07:00:00 +0000");
        rec(&mut xml, "HKQuantityTypeIdentifierHeartRateVariabilitySDNN", "Jo's Apple Watch", "ms", &morning, &morning, if last { "32" } else { "50" });
        rec(&mut xml, "HKQuantityTypeIdentifierRestingHeartRate", "Jo's Apple Watch", "count/min", &morning, &morning, if last { "63" } else { "55" });
        rec(&mut xml, "HKQuantityTypeIdentifierActiveEnergyBurned", "Jo's Apple Watch", "kcal", &morning, &morning, "500");
        let bed = if last { "02:00:00" } else { "23:00:00" };
        let bed_date = if last { date.clone() } else { format!("2026-{m:02}-{:02}", d.saturating_sub(1).max(1)) };
        if i > 0 {
            rec(&mut xml, "HKCategoryTypeIdentifierSleepAnalysis", "Jo's Apple Watch", "", &format!("{bed_date} {bed} +0000"), &format!("{date} 07:00:00 +0000"), "HKCategoryValueSleepAnalysisAsleepCore");
        }
    }
    xml.push_str("</HealthData>\n");
    xml
}

struct Env {
    db: String,
    export: String,
}

fn setup(name: &str) -> Env {
    let dir = std::env::temp_dir().join(format!("vitals-e2e-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let export = dir.join("export.xml");
    std::fs::write(&export, export_xml()).unwrap();
    let db = dir.join("vitals.sqlite");
    let _ = std::fs::remove_file(&db);
    Env { db: db.display().to_string(), export: export.display().to_string() }
}

fn call(env: &Env, calls: &[(&str, Value)]) -> Vec<Value> {
    // Each call is a fresh process, like a client restarting; the database persists.
    calls
        .iter()
        .map(|(name, args)| {
            let out = std::process::Command::new(BIN)
                .env("VITALS_DB", &env.db)
                .env_remove("VITALS_REDACT")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    let req = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                        "params": {"name": name, "arguments": args, "_meta": mcp_kit::testing::stateless_meta()}});
                    writeln!(child.stdin.take().unwrap(), "{req}")?;
                    child.wait_with_output()
                })
                .unwrap();
            let line = String::from_utf8_lossy(&out.stdout).lines().find(|l| l.contains("\"id\":1")).unwrap().to_string();
            serde_json::from_str::<Value>(&line).unwrap()["result"].clone()
        })
        .collect()
}

#[test]
fn import_is_idempotent() {
    let env = setup("idempotent");
    let r = call(&env, &[("import_health_data", json!({"path": env.export})), ("import_health_data", json!({"path": env.export}))]);
    let first = r[0]["structuredContent"]["inserted"].as_u64().unwrap();
    assert!(first > 300, "{}", r[0]);
    assert_eq!(r[1]["structuredContent"]["inserted"], 0);
}

#[test]
fn daily_summary_removes_phone_watch_double_counting() {
    let env = setup("dedup");
    let r = call(&env, &[
        ("import_health_data", json!({"path": env.export})),
        ("daily_summary", json!({"metrics": ["steps"], "from": "2026-10-10", "to": "2026-10-10"})),
    ]);
    let steps = &r[1]["structuredContent"][0];
    assert_eq!(steps["days"][0]["value"], 5700.0, "3 hours × 1900 from the watch, not 11700");
    assert_eq!(steps["double_counted_removed"], 6000.0);
    assert!(r[1]["content"][0]["text"].as_str().unwrap().contains("double-counted"));
}

#[test]
fn readiness_drops_after_a_short_night_and_explains_why() {
    let env = setup("readiness");
    let r = call(&env, &[
        ("import_health_data", json!({"path": env.export})),
        ("readiness", json!({"day": "2026-10-19"})),
        ("readiness", json!({"day": "2026-10-20"})),
    ]);
    let normal = r[1]["structuredContent"]["score"].as_f64().unwrap();
    let rough = r[2]["structuredContent"]["score"].as_f64().unwrap();
    assert!(normal > 75.0, "{}", r[1]);
    assert!(rough < 45.0, "{}", r[2]);
    let text = r[2]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("below your 7-day average"), "{text}");
}

#[test]
fn redaction_hides_device_names() {
    let env = setup("redact");
    let r = call(&env, &[
        ("import_health_data", json!({"path": env.export})),
        ("daily_summary", json!({"metrics": ["steps"], "to": "2026-10-20", "redact": true})),
    ]);
    let body = r[1].to_string();
    assert!(!body.contains("Jo's"), "{body}");
    assert!(body.contains("Source A"));
}

#[test]
fn empty_warehouse_guides_the_user_to_import() {
    let env = setup("empty");
    let r = call(&env, &[("readiness", json!({}))]);
    assert_eq!(r[0]["isError"], true);
    assert!(r[0]["content"][0]["text"].as_str().unwrap().contains("import_health_data"));
}

#[test]
fn lists_five_tools() {
    let env = setup("list");
    // SAFETY: every test passes VITALS_DB explicitly to its own children, so
    // pointing the inherited default at a temp file cannot race with them.
    unsafe { std::env::set_var("VITALS_DB", &env.db) };
    let r = stateless(BIN, &[("tools/list", json!({}))]);
    assert_eq!(r[0]["result"]["tools"].as_array().unwrap().len(), 5);
}
