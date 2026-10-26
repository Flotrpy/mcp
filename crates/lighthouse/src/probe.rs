//! Launch a stdio MCP server and talk to it over both protocol generations.
//!
//! Each generation gets a fresh process: a stateless (`2026-07-28`) server
//! must answer without any handshake, so reusing a process that already ran
//! `initialize` would hide bugs.

use std::{path::PathBuf, process::Stdio, time::Duration};

use serde::Serialize;
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::{Instant, timeout},
};

pub const STATELESS_VERSION: &str = "2026-07-28";
pub const LEGACY_VERSION: &str = "2025-11-25";
/// Stop following `nextCursor` after this many pages.
pub const MAX_PAGES: usize = 20;

/// How to launch the server under test.
#[derive(Debug, Clone, Serialize)]
pub struct Target {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<PathBuf>,
    #[serde(skip)]
    pub timeout: Duration,
}

impl Target {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self { command: command.into(), args, env: Vec::new(), cwd: None, timeout: Duration::from_secs(10) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("could not start `{0}`: {1}")]
    Spawn(String, std::io::Error),
    #[error("{0} timed out after {1} ms")]
    Timeout(String, u128),
    #[error("server closed stdout before answering {0}")]
    Closed(String),
    #[error("{method} returned error {code}: {message}")]
    Rpc { method: String, code: i64, message: String },
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

/// One running server process.
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_id: i64,
    timeout: Duration,
    /// stdout lines that were not JSON-RPC (a common bug: logging to stdout).
    pub stray_output: Vec<String>,
    meta: Option<Value>,
}

impl Session {
    pub async fn start(target: &Target, stateless: bool) -> Result<Self, ProbeError> {
        let mut cmd = Command::new(&target.command);
        cmd.args(&target.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        for (k, v) in &target.env {
            cmd.env(k, v);
        }
        if let Some(dir) = &target.cwd {
            cmd.current_dir(dir);
        }
        let mut child = cmd.spawn().map_err(|e| ProbeError::Spawn(target.command.clone(), e))?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let meta = stateless.then(|| {
            json!({
                "io.modelcontextprotocol/protocolVersion": STATELESS_VERSION,
                "io.modelcontextprotocol/clientInfo": {"name": "lighthouse", "version": env!("CARGO_PKG_VERSION")},
                "io.modelcontextprotocol/clientCapabilities": {}
            })
        });
        Ok(Self {
            child,
            stdin,
            lines: BufReader::new(stdout).lines(),
            next_id: 1,
            timeout: target.timeout,
            stray_output: Vec::new(),
            meta,
        })
    }

    async fn send(&mut self, message: &Value) -> Result<(), ProbeError> {
        let mut line = message.to_string();
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn notify(&mut self, method: &str) -> Result<(), ProbeError> {
        self.send(&json!({"jsonrpc": "2.0", "method": method})).await
    }

    /// Send a request and wait for its response, answering any requests the
    /// server makes of us in the meantime with "method not found".
    pub async fn request(&mut self, method: &str, params: Value) -> Result<(Value, Duration), ProbeError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut params = if params.is_object() { params } else { json!({}) };
        if let Some(meta) = &self.meta {
            params["_meta"] = meta.clone();
        }
        let started = Instant::now();
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})).await?;
        let deadline = started + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let line = match timeout(remaining, self.lines.next_line()).await {
                Err(_) => return Err(ProbeError::Timeout(method.into(), self.timeout.as_millis())),
                Ok(Ok(Some(line))) => line,
                Ok(Ok(None)) => return Err(ProbeError::Closed(method.into())),
                Ok(Err(e)) => return Err(e.into()),
            };
            if line.trim().is_empty() {
                continue;
            }
            let Ok(msg) = serde_json::from_str::<Value>(&line) else {
                if self.stray_output.len() < 5 {
                    self.stray_output.push(line.chars().take(120).collect());
                }
                continue;
            };
            if msg.get("method").is_some() {
                if let Some(req_id) = msg.get("id") {
                    let reply = json!({"jsonrpc": "2.0", "id": req_id, "error": {"code": -32601, "message": "lighthouse does not serve client methods"}});
                    self.send(&reply).await?;
                }
                continue;
            }
            if msg.get("id") != Some(&json!(id)) {
                continue;
            }
            if let Some(err) = msg.get("error") {
                return Err(ProbeError::Rpc {
                    method: method.into(),
                    code: err["code"].as_i64().unwrap_or(0),
                    message: err["message"].as_str().unwrap_or("").to_string(),
                });
            }
            return Ok((msg.get("result").cloned().unwrap_or(Value::Null), started.elapsed()));
        }
    }

    /// `tools/list` with pagination.
    pub async fn list_tools(&mut self) -> Result<(Vec<Value>, Duration), ProbeError> {
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        let mut total = Duration::ZERO;
        for _ in 0..MAX_PAGES {
            let params = match &cursor {
                Some(c) => json!({"cursor": c}),
                None => json!({}),
            };
            let (result, took) = self.request("tools/list", params).await?;
            total += took;
            tools.extend(result["tools"].as_array().cloned().unwrap_or_default());
            cursor = result["nextCursor"].as_str().map(str::to_string);
            if cursor.is_none() {
                break;
            }
        }
        Ok((tools, total))
    }

    pub async fn close(mut self) {
        drop(self.stdin);
        let _ = timeout(Duration::from_secs(2), self.child.wait()).await;
        let _ = self.child.start_kill();
    }
}

/// What one protocol generation looked like.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Generation {
    pub supported: bool,
    pub error: Option<String>,
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    /// Stateless: versions from `server/discover`. Legacy: the negotiated one.
    pub versions: Vec<String>,
    pub handshake_ms: Option<u128>,
    pub list_tools_ms: Option<u128>,
    pub tools: Vec<Value>,
    pub stray_output: Vec<String>,
}

fn ms(d: Duration) -> u128 {
    d.as_millis()
}

/// Probe the `2026-07-28` stateless generation.
pub async fn stateless(target: &Target) -> Generation {
    let mut g = Generation::default();
    let mut s = match Session::start(target, true).await {
        Ok(s) => s,
        Err(e) => {
            g.error = Some(e.to_string());
            return g;
        }
    };
    match s.request("server/discover", json!({})).await {
        Ok((r, took)) => {
            g.handshake_ms = Some(ms(took));
            g.versions = r["supportedVersions"]
                .as_array()
                .map(|v| v.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let info = &r["_meta"]["io.modelcontextprotocol/serverInfo"];
            let info = if info.is_null() { &r["serverInfo"] } else { info };
            g.server_name = info["name"].as_str().map(str::to_string);
            g.server_version = info["version"].as_str().map(str::to_string);
            g.supported = g.versions.iter().any(|v| v == STATELESS_VERSION);
            if !g.supported {
                g.error = Some(format!("server/discover answered but does not list {STATELESS_VERSION}"));
            }
        }
        Err(e) => g.error = Some(e.to_string()),
    }
    if g.supported {
        match s.list_tools().await {
            Ok((tools, took)) => {
                g.tools = tools;
                g.list_tools_ms = Some(ms(took));
            }
            Err(e) => {
                g.supported = false;
                g.error = Some(e.to_string());
            }
        }
    }
    g.stray_output = std::mem::take(&mut s.stray_output);
    s.close().await;
    g
}

/// Probe the `2025-11-25` generation (initialize → initialized → tools/list).
pub async fn legacy(target: &Target) -> Generation {
    let mut g = Generation::default();
    let mut s = match Session::start(target, false).await {
        Ok(s) => s,
        Err(e) => {
            g.error = Some(e.to_string());
            return g;
        }
    };
    let init = json!({
        "protocolVersion": LEGACY_VERSION,
        "capabilities": {},
        "clientInfo": {"name": "lighthouse", "version": env!("CARGO_PKG_VERSION")}
    });
    match s.request("initialize", init).await {
        Ok((r, took)) => {
            g.handshake_ms = Some(ms(took));
            if let Some(v) = r["protocolVersion"].as_str() {
                g.versions = vec![v.to_string()];
            }
            g.server_name = r["serverInfo"]["name"].as_str().map(str::to_string);
            g.server_version = r["serverInfo"]["version"].as_str().map(str::to_string);
            g.supported = true;
        }
        Err(e) => g.error = Some(e.to_string()),
    }
    if g.supported {
        let _ = s.notify("notifications/initialized").await;
        match s.list_tools().await {
            Ok((tools, took)) => {
                g.tools = tools;
                g.list_tools_ms = Some(ms(took));
            }
            Err(e) => {
                g.supported = false;
                g.error = Some(e.to_string());
            }
        }
    }
    g.stray_output = std::mem::take(&mut s.stray_output);
    s.close().await;
    g
}
