//! `handset-mcp`: stdio MCP server to drive Android devices and iOS Simulators.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "handset-mcp {}\n\nDrive Android (adb) and the iOS Simulator (idb, simctl) over MCP (stdio)\nwith compact ref-tagged screens and change-only replies.\n\nAdd to your client:\n  {{\"mcpServers\": {{\"handset\": {{\"command\": \"handset-mcp\"}}}}}}",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    mcp_kit::run_stdio(handset::server::Handset::default()).await
}
