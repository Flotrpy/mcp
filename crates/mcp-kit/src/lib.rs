//! Shared plumbing for the servers in this workspace.
//!
//! Every server uses the same bootstrap ([`run_stdio`]), the same result
//! helpers ([`result`]) and the same token-budget convention ([`budget`]) so
//! that they behave consistently from an agent's point of view.

pub mod budget;
pub mod result;

use rmcp::{ServerHandler, ServiceExt, transport::stdio};

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
