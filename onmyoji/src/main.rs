use onmyoji::{rag::RagStore, server};

use anyhow::{Context, Result};
use knowledge::Knowledge;
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

    // Knowledge DB: ROMHACK_KB env var, or $ROMHACK_ROOT/.knowledge.db
    let kb_path = std::env::var("ROMHACK_KB")
        .unwrap_or_else(|_| format!("{root}/.knowledge.db"));
    let kb = Knowledge::open(&kb_path)
        .with_context(|| format!("failed to open knowledge DB at {kb_path}"))?;

    // Qdrant / RAG: optional — silently skipped if QDRANT_URL is unset or unreachable.
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6333".into());
    let rag = match RagStore::new(&qdrant_url) {
        Ok(r) => {
            tracing::info!("Qdrant connected at {qdrant_url}");
            Some(r)
        }
        Err(e) => {
            tracing::warn!("Qdrant unavailable ({qdrant_url}): {e} — doc tools disabled");
            None
        }
    };

    let server = OnmyojiServer::new_with_rag(root, kb, rag);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
