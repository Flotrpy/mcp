//! `binscope-mcp`: stdio MCP server for read-only binary inspection.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "binscope-mcp {}\n\nRead-only static inspection of ELF, PE and Mach-O files over MCP (stdio).\n\nEnvironment:\n  BINSCOPE_ROOT  only allow files under this directory\n\nAdd to your client:\n  {{\"mcpServers\": {{\"binscope\": {{\"command\": \"binscope-mcp\"}}}}}}",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    mcp_kit::run_stdio(binscope::server::Binscope::default()).await
}
