use onmyoji::server;

use anyhow::{Context, Result};
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use server::OnmyojiServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let root = std::env::var("ROMHACK_ROOT")
        .context("ROMHACK_ROOT is not set. Set it to the absolute path of the romhack-playground repo.")?;

    let server = OnmyojiServer::new(root);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
