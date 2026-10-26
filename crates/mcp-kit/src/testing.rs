//! Test helpers: drive a compiled server binary over stdio with raw
//! JSON-RPC, exactly as a client would.

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

/// Request metadata a stateless (`2026-07-28`) client attaches to every call.
pub fn stateless_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientInfo": {"name": "mcp-kit-tests", "version": "0"},
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

/// Send `(method, params)` requests to `bin` using the stateless protocol and
/// return the responses ordered by request id.
pub fn stateless(bin: &str, requests: &[(&str, Value)]) -> Vec<Value> {
    let lines: Vec<String> = requests
        .iter()
        .enumerate()
        .map(|(i, (method, params))| {
            let mut params = params.clone();
            if !params.is_object() {
                params = json!({});
            }
            params["_meta"] = stateless_meta();
            json!({"jsonrpc": "2.0", "id": i + 1, "method": method, "params": params}).to_string()
        })
        .collect();
    exchange(bin, &lines)
}

/// Run the legacy `2025-11-25` handshake, then send `requests`.
pub fn legacy(bin: &str, requests: &[(&str, Value)]) -> Vec<Value> {
    let mut lines = vec![
        json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "mcp-kit-tests", "version": "0"}}})
        .to_string(),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string(),
    ];
    lines.extend(requests.iter().enumerate().map(|(i, (method, params))| {
        json!({"jsonrpc": "2.0", "id": i + 1, "method": method, "params": params}).to_string()
    }));
    exchange(bin, &lines)
}

/// Convenience: call one tool statelessly and return its `result` object.
pub fn call_tool(bin: &str, name: &str, arguments: Value) -> Value {
    let mut responses = stateless(bin, &[("tools/call", json!({"name": name, "arguments": arguments}))]);
    let response = responses.pop().expect("one response");
    response
        .get("result")
        .cloned()
        .unwrap_or_else(|| panic!("tools/call {name} failed: {response}"))
}

fn exchange(bin: &str, lines: &[String]) -> Vec<Value> {
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("could not start {bin}: {e}"));
    {
        let mut stdin = child.stdin.take().expect("stdin");
        for line in lines {
            writeln!(stdin, "{line}").expect("write request");
        }
    }
    let output = child.wait_with_output().expect("server output");
    let mut responses: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("valid JSON-RPC line"))
        .filter(|v: &Value| v.get("id").is_some())
        .collect();
    responses.sort_by_key(|v| v["id"].as_i64().unwrap_or(-1));
    responses
}
