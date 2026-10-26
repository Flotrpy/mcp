//! `joinery-mcp`: stdio MCP server for woodworking design.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "joinery-mcp {}\n\nA Model Context Protocol server for woodworking: parametric furniture,\ncut lists, sheet nesting and shelf-sag checks. Speak MCP over stdio.\n\nAdd to your client:\n  {{\"mcpServers\": {{\"joinery\": {{\"command\": \"joinery-mcp\"}}}}}}",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    mcp_kit::run_stdio(joinery::server::Joinery::new()).await
}
