//! ROM hacking knowledge base.
//!
//! Two-layer storage:
//!   - SQLite  — structured data: ROM metadata, symbol tables, annotations
//!   - Qdrant  — vector embeddings for semantic search (wired in once an
//!               embedding provider is configured; see src/qdrant.rs)
//!
//! Start here:
//! ```no_run
//! use knowledge::{Knowledge, RomInfo};
//!
//! let kb = Knowledge::open("/path/to/knowledge.db")?;
//! let rom_id = kb.register_rom(&RomInfo {
//!     hash: "abc123".into(),
//!     title: Some("Pokémon Platinum".into()),
//!     platform: "ds".into(),
//!     region: Some("US".into()),
//! })?;
//! # Ok::<(), anyhow::Error>(())
//! ```

mod db;
mod nm;
pub mod qdrant;

pub use db::{Annotation, Knowledge, RomInfo, RomRecord, Symbol};
pub use nm::parse_nm_output;
