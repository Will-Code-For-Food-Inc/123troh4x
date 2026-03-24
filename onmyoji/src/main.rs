mod podman;
mod server;
mod sessions;

use anyhow::Result;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use server::OnmyojiServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let server = OnmyojiServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
