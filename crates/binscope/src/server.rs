//! MCP tool surface for `binscope`: four coarse tools with token budgets.

use std::path::PathBuf;

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::{access, diff, inspect, report, strings};
use mcp_kit::{
    budget::{DEFAULT_MAX_TOKENS, fit},
    result::{structured, tool_error},
};

/// Largest window `hexdump` returns.
pub const MAX_HEXDUMP_BYTES: u64 = 4096;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PathArgs {
    /// Path to an ELF, PE or Mach-O file.
    pub path: String,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Sections,
    Libraries,
    Imports,
    Exports,
    Strings,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InspectArgs {
    /// Path to an ELF, PE or Mach-O file.
    pub path: String,
    /// Which list to return.
    pub view: View,
    /// Case-insensitive substring filter applied to names / text.
    pub filter: Option<String>,
    /// Strings only: minimum run length. Default 6.
    pub min_length: Option<usize>,
    /// Approximate output budget. Default 2000; lists are trimmed by dropping
    /// the least interesting entries, never mid-entry.
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HexArgs {
    /// Path to any file.
    pub path: String,
    /// Byte offset into the file.
    pub offset: u64,
    /// Bytes to show, at most 4096. Default 256.
    pub length: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiffArgs {
    /// The earlier build.
    pub old_path: String,
    /// The later build.
    pub new_path: String,
    /// Approximate output budget for the symbol lists. Default 2000.
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Binscope {
    root: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

impl Default for Binscope {
    fn default() -> Self {
        Self::new(access::root_from_env())
    }
}

type Loaded = (Vec<u8>, inspect::Binary);

impl Binscope {
    fn read(&self, path: &str) -> Result<Vec<u8>, CallToolResult> {
        let real = access::resolve(path, self.root.as_deref()).map_err(|e| tool_error(e.to_string()))?;
        std::fs::read(&real).map_err(|e| tool_error(format!("Could not read {}: {e}", real.display())))
    }

    fn load(&self, path: &str) -> Result<Loaded, CallToolResult> {
        let bytes = self.read(path)?;
        let bin = inspect::parse(&bytes).map_err(|e| tool_error(format!("{path}: {e}")))?;
        Ok((bytes, bin))
    }
}

fn matches(filter: &Option<String>, text: &str) -> bool {
    filter.as_ref().is_none_or(|f| text.to_ascii_lowercase().contains(&f.to_ascii_lowercase()))
}

macro_rules! try_tool {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(err) => return Ok(err),
        }
    };
}

#[tool_router]
impl Binscope {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root, tool_router: Self::tool_router() }
    }

    #[tool(
        description = "First look at an executable or library: format, architecture, entry point, stripped or not, counts, a whole-file entropy map, the largest sections and suggested next calls. Read-only.",
        annotations(title = "Binary overview", read_only_hint = true, open_world_hint = false)
    )]
    pub fn overview(&self, Parameters(a): Parameters<PathArgs>) -> Result<CallToolResult, ErrorData> {
        let (bytes, bin) = try_tool!(self.load(&a.path));
        let o = report::overview(&bin, &bytes);
        let summary = format!(
            "{} {} {}-bit, {} bytes, {} sections, {} imports, {} exports. Entropy: {}",
            o.format,
            o.architecture,
            o.bits,
            o.file_size,
            o.counts.sections,
            o.counts.imports,
            o.counts.exports,
            o.entropy_map
        );
        structured(summary, &o)
    }

    #[tool(
        description = "List one aspect of a binary — sections, libraries, imports, exports or strings — filtered and trimmed to a token budget, most informative entries first.",
        annotations(title = "Inspect binary", read_only_hint = true, open_world_hint = false)
    )]
    pub fn inspect(&self, Parameters(a): Parameters<InspectArgs>) -> Result<CallToolResult, ErrorData> {
        let (bytes, bin) = try_tool!(self.load(&a.path));
        let budget = a.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
        let value = match a.view {
            View::Sections => {
                let mut items: Vec<_> =
                    bin.sections.into_iter().filter(|s| matches(&a.filter, &s.name)).collect();
                items.sort_by_key(|s| std::cmp::Reverse(s.file_size));
                serde_json::to_value(fit(items, budget))
            }
            View::Libraries => serde_json::to_value(fit(
                bin.libraries.into_iter().filter(|l| matches(&a.filter, l)).collect(),
                budget,
            )),
            View::Imports => {
                let items: Vec<_> = bin
                    .imports
                    .into_iter()
                    .filter(|i| {
                        matches(&a.filter, &i.name)
                            || i.library.as_deref().is_some_and(|l| matches(&a.filter, l))
                    })
                    .collect();
                serde_json::to_value(fit(items, budget))
            }
            View::Exports => serde_json::to_value(fit(
                bin.exports.into_iter().filter(|e| matches(&a.filter, e)).collect(),
                budget,
            )),
            View::Strings => {
                let found: Vec<_> = strings::extract(&bytes, &bin.sections, a.min_length.unwrap_or(6))
                    .into_iter()
                    .filter(|s| matches(&a.filter, &s.text))
                    .collect();
                serde_json::to_value(fit(found, budget))
            }
        }
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        let shown = value["items"].as_array().map_or(0, Vec::len);
        let omitted = value["omitted"].as_u64().unwrap_or(0);
        let summary = if omitted > 0 {
            format!(
                "{shown} shown, {omitted} more omitted to stay under ~{budget} tokens; narrow with `filter` or raise `max_tokens`."
            )
        } else {
            format!("{shown} entries.")
        };
        structured(summary, &value)
    }

    #[tool(
        description = "Hex and ASCII view of up to 4096 bytes at a file offset (e.g. a section start from overview). Read-only.",
        annotations(title = "Hex dump", read_only_hint = true, open_world_hint = false)
    )]
    pub fn hexdump(&self, Parameters(a): Parameters<HexArgs>) -> Result<CallToolResult, ErrorData> {
        let bytes = try_tool!(self.read(&a.path));
        let len = a.length.unwrap_or(256).min(MAX_HEXDUMP_BYTES);
        let start = a.offset.min(bytes.len() as u64) as usize;
        let end = (start + len as usize).min(bytes.len());
        if start >= bytes.len() {
            return Ok(tool_error(format!(
                "Offset {} is past the end of the file ({} bytes).",
                a.offset,
                bytes.len()
            )));
        }
        let dump = report::hexdump(&bytes[start..end], start as u64);
        structured(dump.clone(), &json!({"offset": start, "length": end - start, "dump": dump}))
    }

    #[tool(
        description = "Compare two builds of a program: size change, sections added/removed/resized, entropy jumps, linked libraries, imports and exports, with headline highlights first.",
        annotations(title = "Diff two binaries", read_only_hint = true, open_world_hint = false)
    )]
    pub fn diff(&self, Parameters(a): Parameters<DiffArgs>) -> Result<CallToolResult, ErrorData> {
        let (_, old) = try_tool!(self.load(&a.old_path));
        let (_, new) = try_tool!(self.load(&a.new_path));
        let d = diff::diff(&old, &new);
        let budget = a.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
        let summary = if d.highlights.is_empty() {
            format!("Size delta {:+} bytes; no headline changes.", d.size_delta)
        } else {
            format!("Size delta {:+} bytes. {}", d.size_delta, d.highlights.join(" "))
        };
        let mut value =
            serde_json::to_value(&d).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        // Long symbol lists are the bulk of a diff; fit each to a share of the budget.
        for key in ["imports", "exports"] {
            for side in ["added", "removed"] {
                let items: Vec<String> = serde_json::from_value(value[key][side].take()).unwrap_or_default();
                value[key][side] = serde_json::to_value(fit(items, budget / 4)).unwrap_or_default();
            }
        }
        structured(summary, &value)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Binscope {
    fn get_info(&self) -> ServerConfig {
        mcp_kit::server_config(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            "Binscope — static binary inspection",
            "Start with overview; it suggests the next calls. Use inspect for one list at a time \
             (sections, libraries, imports, exports, strings) and diff to compare two builds. \
             Everything is read-only; nothing is executed. Long lists honour max_tokens.",
        )
    }
}
