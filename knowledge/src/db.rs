//! SQLite-backed knowledge store.

use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::{Connection, params};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RomInfo {
    pub hash: String,
    pub title: Option<String>,
    pub platform: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RomRecord {
    pub id: i64,
    pub hash: String,
    pub title: Option<String>,
    pub platform: String,
    pub region: Option<String>,
}

/// A symbol extracted from an ELF via nm.
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Virtual address.
    pub address: u32,
    /// Size in bytes (available with `nm --print-size`).
    pub size: Option<u32>,
    /// nm type character: T/t (text), D/d (data), B/b (bss), etc.
    pub kind: String,
    /// Demangled symbol name.
    pub name: String,
    /// Provenance tag: "decomp", "user", "validated".
    pub source: String,
}

impl Symbol {
    pub fn from_decomp(address: u32, size: Option<u32>, kind: impl Into<String>, name: impl Into<String>) -> Self {
        Self { address, size, kind: kind.into(), name: name.into(), source: "decomp".into() }
    }
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub address: u32,
    pub description: String,
    pub validated: bool,
    pub source: String,
}

// ── Knowledge ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Knowledge {
    conn: Arc<Mutex<Connection>>,
}

impl Knowledge {
    /// Open (or create) a knowledge database at the given path.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open knowledge db at {path}"))?;
        let kb = Self { conn: Arc::new(Mutex::new(conn)) };
        kb.migrate()?;
        Ok(kb)
    }

    /// Open an in-memory database (useful for tests).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("failed to open in-memory knowledge db")?;
        let kb = Self { conn: Arc::new(Mutex::new(conn)) };
        kb.migrate()?;
        Ok(kb)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(SCHEMA).context("schema migration failed")?;
        Ok(())
    }

    // ── ROM management ────────────────────────────────────────────────────────

    /// Insert a ROM record. Returns the row id.
    /// If the ROM hash already exists, returns its existing id.
    pub fn register_rom(&self, info: &RomInfo) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO roms (hash, title, platform, region) VALUES (?1, ?2, ?3, ?4)",
            params![info.hash, info.title, info.platform, info.region],
        ).context("insert rom")?;
        let id: i64 = conn.query_row(
            "SELECT id FROM roms WHERE hash = ?1",
            params![info.hash],
            |row| row.get(0),
        ).context("fetch rom id")?;
        Ok(id)
    }

    pub fn get_rom_by_hash(&self, hash: &str) -> Result<Option<RomRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, hash, title, platform, region FROM roms WHERE hash = ?1"
        )?;
        let mut rows = stmt.query(params![hash])?;
        if let Some(row) = rows.next()? {
            Ok(Some(RomRecord {
                id:       row.get(0)?,
                hash:     row.get(1)?,
                title:    row.get(2)?,
                platform: row.get(3)?,
                region:   row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ── Symbol management ─────────────────────────────────────────────────────

    /// Bulk-insert symbols. Uses INSERT OR IGNORE so re-running is safe.
    /// Returns the number of newly inserted rows.
    pub fn register_symbols(&self, rom_id: i64, symbols: &[Symbol]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().context("begin transaction")?;
        let mut inserted = 0usize;
        for sym in symbols {
            let n = tx.execute(
                "INSERT OR IGNORE INTO symbols (rom_id, address, size, kind, name, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![rom_id, sym.address, sym.size, sym.kind, sym.name, sym.source],
            ).context("insert symbol")?;
            inserted += n;
        }
        tx.commit().context("commit transaction")?;
        Ok(inserted)
    }

    /// Look up the symbol at exactly this address (if any).
    pub fn lookup_symbol(&self, rom_id: i64, address: u32) -> Result<Option<Symbol>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT address, size, kind, name, source FROM symbols
             WHERE rom_id = ?1 AND address = ?2
             ORDER BY id LIMIT 1"
        )?;
        let mut rows = stmt.query(params![rom_id, address])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Symbol {
                address: row.get(0)?,
                size:    row.get(1)?,
                kind:    row.get(2)?,
                name:    row.get(3)?,
                source:  row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Look up all symbols whose name contains `pattern` (case-insensitive substring).
    pub fn lookup_symbols_by_name(&self, rom_id: i64, pattern: &str) -> Result<Vec<Symbol>> {
        let conn = self.conn.lock().unwrap();
        let like = format!("%{pattern}%");
        let mut stmt = conn.prepare(
            "SELECT address, size, kind, name, source FROM symbols
             WHERE rom_id = ?1 AND name LIKE ?2
             ORDER BY address"
        )?;
        let rows = stmt.query_map(params![rom_id, like], |row| {
            Ok(Symbol {
                address: row.get(0)?,
                size:    row.get(1)?,
                kind:    row.get(2)?,
                name:    row.get(3)?,
                source:  row.get(4)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("query symbols by name")
    }

    /// Return all symbols with address in `[start, end)`, ordered by address.
    pub fn symbols_in_range(&self, rom_id: i64, start: u32, end: u32) -> Result<Vec<Symbol>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT address, size, kind, name, source FROM symbols
             WHERE rom_id = ?1 AND address >= ?2 AND address < ?3
             ORDER BY address"
        )?;
        let rows = stmt.query_map(params![rom_id, start, end], |row| {
            Ok(Symbol {
                address: row.get(0)?,
                size:    row.get(1)?,
                kind:    row.get(2)?,
                name:    row.get(3)?,
                source:  row.get(4)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("query symbols in range")
    }

    // ── Annotation management ─────────────────────────────────────────────────

    /// Add an annotation for an address. Returns the new annotation id.
    pub fn add_annotation(&self, rom_id: i64, ann: &Annotation) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO annotations (rom_id, address, description, validated, source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![rom_id, ann.address, ann.description, ann.validated as i32, ann.source],
        ).context("insert annotation")?;
        Ok(conn.last_insert_rowid())
    }

    /// Fetch all annotations for a given address.
    pub fn get_annotations(&self, rom_id: i64, address: u32) -> Result<Vec<Annotation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT address, description, validated, source FROM annotations
             WHERE rom_id = ?1 AND address = ?2
             ORDER BY id"
        )?;
        let rows = stmt.query_map(params![rom_id, address], |row| {
            Ok(Annotation {
                address:     row.get(0)?,
                description: row.get(1)?,
                validated:   row.get::<_, i32>(2)? != 0,
                source:      row.get(3)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().context("query annotations")
    }
}

// ── Schema ────────────────────────────────────────────────────────────────────

const SCHEMA: &str = "
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS roms (
    id       INTEGER PRIMARY KEY,
    hash     TEXT    NOT NULL UNIQUE,
    title    TEXT,
    platform TEXT    NOT NULL,
    region   TEXT,
    added_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS symbols (
    id       INTEGER PRIMARY KEY,
    rom_id   INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    address  INTEGER NOT NULL,
    size     INTEGER,
    kind     TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    source   TEXT    NOT NULL DEFAULT 'decomp',
    added_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_symbols_unique  ON symbols(rom_id, address, name);
CREATE        INDEX IF NOT EXISTS idx_symbols_address ON symbols(rom_id, address);
CREATE        INDEX IF NOT EXISTS idx_symbols_name    ON symbols(rom_id, name);

CREATE TABLE IF NOT EXISTS annotations (
    id          INTEGER PRIMARY KEY,
    rom_id      INTEGER NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    address     INTEGER NOT NULL,
    description TEXT    NOT NULL,
    validated   INTEGER NOT NULL DEFAULT 0,
    source      TEXT    NOT NULL DEFAULT 'user',
    added_at    INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_annotations_address ON annotations(rom_id, address);
";

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> Knowledge {
        Knowledge::open_in_memory().unwrap()
    }

    fn platinum_rom() -> RomInfo {
        RomInfo {
            hash: "abc123".into(),
            title: Some("Pokémon Platinum".into()),
            platform: "ds".into(),
            region: Some("US".into()),
        }
    }

    // ── ROM ───────────────────────────────────────────────────────────────────

    #[test]
    fn register_and_fetch_rom() {
        let kb = kb();
        let id = kb.register_rom(&platinum_rom()).unwrap();
        assert!(id > 0);
        let rec = kb.get_rom_by_hash("abc123").unwrap().unwrap();
        assert_eq!(rec.id, id);
        assert_eq!(rec.platform, "ds");
        assert_eq!(rec.title.as_deref(), Some("Pokémon Platinum"));
    }

    #[test]
    fn register_rom_is_idempotent() {
        let kb = kb();
        let id1 = kb.register_rom(&platinum_rom()).unwrap();
        let id2 = kb.register_rom(&platinum_rom()).unwrap();
        assert_eq!(id1, id2, "re-registering same hash must return same id");
    }

    #[test]
    fn get_unknown_rom_returns_none() {
        let kb = kb();
        assert!(kb.get_rom_by_hash("nope").unwrap().is_none());
    }

    // ── Symbols ───────────────────────────────────────────────────────────────

    fn syms() -> Vec<Symbol> {
        vec![
            Symbol::from_decomp(0x0200_0000, Some(0x10), "T", "main"),
            Symbol::from_decomp(0x0200_0010, Some(0x08), "T", "init"),
            Symbol::from_decomp(0x0200_0018, None,       "t", "helper"),
            Symbol::from_decomp(0x0200_1000, Some(0x04), "D", "gCounter"),
        ]
    }

    #[test]
    fn register_and_lookup_symbol_by_address() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        let sym = kb.lookup_symbol(rom_id, 0x0200_0000).unwrap().unwrap();
        assert_eq!(sym.name, "main");
        assert_eq!(sym.kind, "T");
        assert_eq!(sym.size, Some(0x10));
    }

    #[test]
    fn lookup_symbol_missing_address_returns_none() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        assert!(kb.lookup_symbol(rom_id, 0xDEAD_BEEF).unwrap().is_none());
    }

    #[test]
    fn register_symbols_count() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        let n = kb.register_symbols(rom_id, &syms()).unwrap();
        assert_eq!(n, 4);
    }

    #[test]
    fn register_symbols_is_idempotent() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        let n1 = kb.register_symbols(rom_id, &syms()).unwrap();
        let n2 = kb.register_symbols(rom_id, &syms()).unwrap();
        assert_eq!(n1, 4);
        assert_eq!(n2, 0, "second insert should be no-ops");
    }

    #[test]
    fn lookup_symbols_by_name_substring() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        let results = kb.lookup_symbols_by_name(rom_id, "init").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "init");
    }

    #[test]
    fn lookup_symbols_by_name_case_insensitive() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        // LIKE in SQLite is case-insensitive for ASCII
        let results = kb.lookup_symbols_by_name(rom_id, "MAIN").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn lookup_symbols_by_name_no_match() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        let results = kb.lookup_symbols_by_name(rom_id, "xyzzy_no_match").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn symbols_in_range() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        // Range covering first two symbols
        let results = kb.symbols_in_range(rom_id, 0x0200_0000, 0x0200_0018).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "main");
        assert_eq!(results[1].name, "init");
    }

    #[test]
    fn symbols_in_range_empty() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        kb.register_symbols(rom_id, &syms()).unwrap();

        let results = kb.symbols_in_range(rom_id, 0xFFFF_0000, 0xFFFF_FFFF).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn symbols_isolated_by_rom_id() {
        let kb = kb();
        let id1 = kb.register_rom(&RomInfo {
            hash: "rom1".into(), title: None, platform: "ds".into(), region: None,
        }).unwrap();
        let id2 = kb.register_rom(&RomInfo {
            hash: "rom2".into(), title: None, platform: "gba".into(), region: None,
        }).unwrap();

        kb.register_symbols(id1, &syms()).unwrap();

        // id2 should see nothing
        assert!(kb.lookup_symbol(id2, 0x0200_0000).unwrap().is_none());
        assert!(kb.lookup_symbols_by_name(id2, "main").unwrap().is_empty());
    }

    // ── Annotations ───────────────────────────────────────────────────────────

    #[test]
    fn add_and_fetch_annotation() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        let ann = Annotation {
            address:     0x0200_0000,
            description: "Entry point — sets up stack and branches to main".into(),
            validated:   true,
            source:      "user".into(),
        };
        let ann_id = kb.add_annotation(rom_id, &ann).unwrap();
        assert!(ann_id > 0);

        let fetched = kb.get_annotations(rom_id, 0x0200_0000).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].description, ann.description);
        assert!(fetched[0].validated);
    }

    #[test]
    fn annotations_empty_for_unknown_address() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        let anns = kb.get_annotations(rom_id, 0xDEAD_BEEF).unwrap();
        assert!(anns.is_empty());
    }

    #[test]
    fn multiple_annotations_per_address() {
        let kb = kb();
        let rom_id = kb.register_rom(&platinum_rom()).unwrap();
        for i in 0..3 {
            kb.add_annotation(rom_id, &Annotation {
                address:     0x1000,
                description: format!("note {i}"),
                validated:   false,
                source:      "user".into(),
            }).unwrap();
        }
        let anns = kb.get_annotations(rom_id, 0x1000).unwrap();
        assert_eq!(anns.len(), 3);
    }
}
