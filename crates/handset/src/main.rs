//! `handset-mcp`: stdio MCP server to drive Android devices and iOS Simulators.

use mcp_kit::cli::{About, handle};

const ABOUT: About = About {
    name: "handset",
    version: env!("CARGO_PKG_VERSION"),
    summary: "Drive Android (adb) and the iOS Simulator (idb, simctl) over MCP with compact ref-tagged screens and change-only replies.",
    env: &[],
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(text) = handle(&ABOUT, std::env::args().skip(1)) {
        print!("{text}");
        return Ok(());
    }
    mcp_kit::run_stdio(handset::server::Handset::default()).await
}
