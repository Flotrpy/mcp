//! `vitals-mcp`: stdio MCP server for a local health-data warehouse.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "vitals-mcp {}\n\nLocal-first health warehouse over MCP (stdio).\n\nEnvironment:\n  VITALS_DB      SQLite file (default {})\n  VITALS_REDACT  set to 1 to hide source names in every result\n\nAdd to your client:\n  {{\"mcpServers\": {{\"vitals\": {{\"command\": \"vitals-mcp\"}}}}}}",
            env!("CARGO_PKG_VERSION"),
            vitals::server::default_db_path().display()
        );
        return Ok(());
    }
    let server = vitals::server::Vitals::open(&vitals::server::default_db_path())?;
    mcp_kit::run_stdio(server).await
}
