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
pub use rom_id::hash_rom_file;

mod rom_id {
    use anyhow::{Context, Result};

    /// Compute a CRC32 hash of a ROM file. Returns a zero-padded 8-char hex string.
    ///
    /// This is the primary ROM identity key used throughout the knowledge store.
    /// Matches No-Intro naming convention (CRC32 in uppercase hex is standard
    /// in their DAT files, but we store lowercase for consistency).
    pub fn hash_rom_file(path: &str) -> Result<String> {
        let data = std::fs::read(path)
            .with_context(|| format!("failed to read ROM file: {path}"))?;
        let crc = crc32fast::hash(&data);
        Ok(format!("{crc:08x}"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn hash_is_8_hex_chars() {
            // Write a small temp file and hash it
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("test.bin");
            std::fs::write(&path, b"\x00\x01\x02\x03").unwrap();
            let h = hash_rom_file(path.to_str().unwrap()).unwrap();
            assert_eq!(h.len(), 8);
            assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn same_content_same_hash() {
            let dir = tempfile::tempdir().unwrap();
            let p1 = dir.path().join("a.bin");
            let p2 = dir.path().join("b.bin");
            std::fs::write(&p1, b"hello rom").unwrap();
            std::fs::write(&p2, b"hello rom").unwrap();
            assert_eq!(
                hash_rom_file(p1.to_str().unwrap()).unwrap(),
                hash_rom_file(p2.to_str().unwrap()).unwrap(),
            );
        }

        #[test]
        fn different_content_different_hash() {
            let dir = tempfile::tempdir().unwrap();
            let p1 = dir.path().join("a.bin");
            let p2 = dir.path().join("b.bin");
            std::fs::write(&p1, b"rom one").unwrap();
            std::fs::write(&p2, b"rom two").unwrap();
            assert_ne!(
                hash_rom_file(p1.to_str().unwrap()).unwrap(),
                hash_rom_file(p2.to_str().unwrap()).unwrap(),
            );
        }

        #[test]
        fn missing_file_returns_error() {
            assert!(hash_rom_file("/nonexistent/path/to/rom.gba").is_err());
        }
    }
}

