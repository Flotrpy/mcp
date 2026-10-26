//! `joinery-mcp`: stdio MCP server for woodworking design.

use mcp_kit::cli::{About, handle};

const ABOUT: About = About {
    name: "joinery",
    version: env!("CARGO_PKG_VERSION"),
    summary: "Woodworking over MCP: parametric furniture, cut lists, sheet nesting and shelf-sag checks.",
    env: &[],
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(text) = handle(&ABOUT, std::env::args().skip(1)) {
        print!("{text}");
        return Ok(());
    }
    mcp_kit::run_stdio(joinery::server::Joinery::new()).await
}
