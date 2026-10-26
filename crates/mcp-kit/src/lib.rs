//! Shared plumbing for the servers in this workspace.
//!
//! Every server uses the same bootstrap ([`run_stdio`]), the same result
//! helpers ([`result`]) and the same token-budget convention ([`budget`]) so
//! that they behave consistently from an agent's point of view.

pub mod budget;
pub mod result;
pub mod testing;

use rmcp::{
    ServerHandler, ServiceExt,
    model::{Implementation, ServerCapabilities, ServerConfig},
    transport::stdio,
};

/// Build the `ServerConfig` every server in the workspace returns from
/// `get_info`: tools enabled, the crate's own name and version (not the SDK's),
/// a human title and instructions that tell the model how to use the tools.
pub fn server_config(name: &str, version: &str, title: &str, instructions: &str) -> ServerConfig {
    ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(
            Implementation::new(name, version)
                .with_title(title)
                .with_website_url("https://github.com/Flotrpy/mcp"),
        )
        .with_instructions(instructions)
}

/// Serve `handler` over stdio until the client disconnects.
///
/// Logs go to stderr (stdout carries the protocol). Set `RUST_LOG=debug` for
/// verbose output.
pub async fn run_stdio<H>(handler: H) -> anyhow::Result<()>
where
    H: ServerHandler,
{
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .try_init()
        .ok();
    let service = handler.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
