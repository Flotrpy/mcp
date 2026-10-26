//! MCP tool surface for `lighthouse`.

use std::{collections::HashMap, path::PathBuf, time::Duration};

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    audit,
    lock::{self, Lockfile},
    probe::{self, Target},
};
use mcp_kit::result::{structured, tool_error};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TargetArgs {
    /// Executable that starts the server, e.g. `npx` or `/usr/local/bin/my-mcp`.
    pub command: String,
    /// Arguments, e.g. `["-y", "@scope/server"]`.
    #[serde(default)]
    pub args: Vec<String>,
    /// Extra environment variables for the server process.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory for the server process.
    pub cwd: Option<String>,
    /// Per-request timeout in seconds. Default 10.
    pub timeout_seconds: Option<u64>,
}

impl TargetArgs {
    fn target(&self) -> Target {
        let mut t = Target::new(self.command.clone(), self.args.clone());
        t.env = self.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        t.cwd = self.cwd.as_ref().map(PathBuf::from);
        t.timeout = Duration::from_secs(self.timeout_seconds.unwrap_or(10).clamp(1, 120));
        t
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LockArgs {
    #[serde(flatten)]
    pub target: TargetArgs,
    /// Where the lockfile lives. Default `mcp.lock` in the working directory.
    pub lock_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LintArgs {
    /// A `tools/list` result, or just its `tools` array.
    pub tools: Value,
}

#[derive(Debug, Clone)]
pub struct Lighthouse {
    tool_router: ToolRouter<Self>,
}

impl Default for Lighthouse {
    fn default() -> Self {
        Self::new()
    }
}

async fn current_lock(target: &Target) -> Result<Lockfile, String> {
    let s = probe::stateless(target).await;
    let g = if s.supported { s } else { probe::legacy(target).await };
    if !g.supported {
        return Err(g.error.unwrap_or_else(|| "server did not answer".into()));
    }
    Ok(Lockfile::from_tools(g.server_name, g.server_version, &g.tools))
}

#[tool_router]
impl Lighthouse {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(
        description = "Launch a stdio MCP server and audit it: 2026-07-28 stateless and 2025-11-25 handshake support, tool-definition lint, per-tool token cost, and an A–F grade with the reasons points were lost. Starts the given command.",
        annotations(title = "Audit an MCP server", read_only_hint = true, open_world_hint = true)
    )]
    pub async fn audit_server(
        &self,
        Parameters(a): Parameters<TargetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let report = audit::run(&a.target()).await;
        let summary = format!(
            "Grade {} ({}/100) for {} — migration: {}, {} tools, ~{} tokens per turn, {} finding(s).",
            report.score.grade,
            report.score.points,
            report.server_name.as_deref().unwrap_or(&a.command),
            report.migration,
            report.tool_count,
            report.cost.total_tokens,
            report.findings.len()
        );
        structured(summary, &report)
    }

    #[tool(
        description = "Write mcp.lock pinning a server's current tool names, descriptions and schema hashes. Commit the file; verify_server then catches silent changes.",
        annotations(
            title = "Pin tools to mcp.lock",
            read_only_hint = false,
            destructive_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn pin_server(&self, Parameters(a): Parameters<LockArgs>) -> Result<CallToolResult, ErrorData> {
        let lockfile = match current_lock(&a.target.target()).await {
            Ok(l) => l,
            Err(e) => return Ok(tool_error(format!("Could not read tools: {e}"))),
        };
        let path = a.lock_path.unwrap_or_else(|| lock::DEFAULT_PATH.into());
        if let Err(e) = std::fs::write(&path, lockfile.to_pretty_json()) {
            return Ok(tool_error(format!("Could not write {path}: {e}")));
        }
        structured(
            format!("Pinned {} tool(s) to {path}.", lockfile.tools.len()),
            &json!({"path": path, "lockfile": lockfile}),
        )
    }

    #[tool(
        description = "Compare a server's live tools against mcp.lock and report tools added, removed or changed (description, inputSchema, annotations), showing old and new descriptions.",
        annotations(title = "Verify against mcp.lock", read_only_hint = true, open_world_hint = true)
    )]
    pub async fn verify_server(
        &self,
        Parameters(a): Parameters<LockArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = a.lock_path.clone().unwrap_or_else(|| lock::DEFAULT_PATH.into());
        let pinned: Lockfile = match std::fs::read_to_string(&path).map(|s| serde_json::from_str(&s)) {
            Ok(Ok(l)) => l,
            Ok(Err(e)) => return Ok(tool_error(format!("{path} is not a valid lockfile: {e}"))),
            Err(_) => return Ok(tool_error(format!("No lockfile at {path}; run pin_server first."))),
        };
        let current = match current_lock(&a.target.target()).await {
            Ok(l) => l,
            Err(e) => return Ok(tool_error(format!("Could not read tools: {e}"))),
        };
        let drift = lock::compare(&pinned, &current);
        structured(drift.summary(), &drift)
    }

    #[tool(
        description = "Lint tool definitions you already have (a pasted tools/list result) without launching anything; returns findings and token cost.",
        annotations(title = "Lint tool definitions", read_only_hint = true, open_world_hint = false)
    )]
    pub fn lint_tools(&self, Parameters(a): Parameters<LintArgs>) -> Result<CallToolResult, ErrorData> {
        let tools = match &a.tools {
            Value::Array(t) => t.clone(),
            Value::Object(o) => o.get("tools").and_then(Value::as_array).cloned().unwrap_or_default(),
            _ => Vec::new(),
        };
        if tools.is_empty() {
            return Ok(tool_error("Pass a tools/list result or its `tools` array in `tools`."));
        }
        let (findings, cost) = audit::offline(&tools);
        structured(
            format!(
                "{} tool(s), {} finding(s), ~{} tokens per turn.",
                tools.len(),
                findings.len(),
                cost.total_tokens
            ),
            &json!({"findings": findings, "cost": cost}),
        )
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Lighthouse {
    fn get_info(&self) -> ServerConfig {
        mcp_kit::server_config(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            "Lighthouse — MCP server auditor",
            "audit_server grades a stdio MCP server end to end. pin_server writes mcp.lock and \
             verify_server reports drift against it. lint_tools checks pasted definitions offline.",
        )
    }
}
