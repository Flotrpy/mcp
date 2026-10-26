//! `vitals-mcp`: stdio MCP server for a local health-data warehouse.

use mcp_kit::cli::{About, handle};

const ABOUT: About = About {
    name: "vitals",
    version: env!("CARGO_PKG_VERSION"),
    summary: "A local-first health warehouse over MCP: Apple Health and CSV import, cross-device de-duplication, explainable readiness.",
    env: &[
        ("VITALS_DB", "SQLite file (default ~/.local/share/vitals/vitals.sqlite)"),
        ("VITALS_REDACT", "set to 1 to hide source names in every result"),
    ],
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(text) = handle(&ABOUT, std::env::args().skip(1)) {
        print!("{text}");
        return Ok(());
    }
    let server = vitals::server::Vitals::open(&vitals::server::default_db_path())?;
    mcp_kit::run_stdio(server).await
}
