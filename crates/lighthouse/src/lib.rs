//! Audit MCP servers the way Lighthouse audits web pages.
//!
//! `lighthouse` launches a stdio MCP server, speaks to it over both protocol
//! generations, lints its tool definitions, estimates what they cost in
//! context tokens, grades the result and can pin the tool surface to an
//! `mcp.lock` file so silent changes are caught in CI.

pub mod canonical;
