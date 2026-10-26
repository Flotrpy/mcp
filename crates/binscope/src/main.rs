//! `binscope-mcp`: stdio MCP server for read-only binary inspection.

use mcp_kit::cli::{About, handle};

const ABOUT: About = About {
    name: "binscope",
    version: env!("CARGO_PKG_VERSION"),
    summary: "Read-only static inspection of ELF, PE and Mach-O files over MCP. Nothing is executed.",
    env: &[("BINSCOPE_ROOT", "only allow files under this directory")],
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(text) = handle(&ABOUT, std::env::args().skip(1)) {
        print!("{text}");
        return Ok(());
    }
    mcp_kit::run_stdio(binscope::server::Binscope::default()).await
}
