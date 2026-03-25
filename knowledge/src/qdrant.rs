//! Qdrant vector store integration.
//!
//! # Current state
//!
//! Infrastructure is in place (container, collection naming, point schema) but
//! the Rust client is not yet wired in.  Qdrant is most useful once we have a
//! real embedding provider — a 1536-dim OpenAI or local model call for each
//! symbol name + description.  Until then, the SQLite layer (`Knowledge`) is
//! the sole query interface.
//!
//! # Planned API (not yet implemented)
//!
//! ```text
//! let store = VectorStore::connect("http://localhost:6334", "romhack_ds").await?;
//! store.ensure_collection(1536).await?;
//!
//! // After embedding symbol name+description with your provider:
//! store.upsert_symbol(symbol_sqlite_id, embedding_vec, json!({
//!     "rom_id":  rom_id,
//!     "address": 0x0200_0000,
//!     "name":    "main",
//!     "kind":    "T",
//! })).await?;
//!
//! // Semantic search:
//! let hits = store.search(&query_embedding, 10).await?;
//! ```
//!
//! # Container
//!
//! Start Qdrant with:
//! ```text
//! make run-qdrant
//! ```
//!
//! Collections are named `romhack_<platform>` (e.g. `romhack_ds`, `romhack_gba`).
//! Each point:
//!   - id      = SQLite symbol row id (u64)
//!   - vector  = embedding(name + description), dim=1536
//!   - payload = { rom_id, address, name, kind, source }
//!
//! # Adding the client
//!
//! When ready to wire in:
//! 1. Add to knowledge/Cargo.toml:
//!    ```toml
//!    [dependencies]
//!    qdrant-client = "1.10"
//!    tokio = { version = "1", features = ["full"] }
//!    ```
//! 2. Implement `VectorStore` below using `qdrant_client::Qdrant`.
//! 3. Add `QDRANT_URL` env var support to `Knowledge::open_with_qdrant()`.

/// Collection name for a given platform.
pub fn collection_name(platform: &str) -> String {
    format!("romhack_{platform}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_name_format() {
        assert_eq!(collection_name("ds"),  "romhack_ds");
        assert_eq!(collection_name("gba"), "romhack_gba");
        assert_eq!(collection_name("gen"), "romhack_gen");
    }
}
