mod podman;
mod server;
mod sessions;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let svc = server::RomhackServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {e:?}"))?;

    svc.waiting().await?;
    Ok(())
}
