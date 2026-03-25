//! Knowledge crate integration tests.
//!
//! These tests use a real (temp-file) SQLite database rather than in-memory,
//! verifying open/close/reopen behaviour.  No container or Qdrant needed.

use knowledge::{Knowledge, RomInfo, Symbol, parse_nm_output};

fn temp_db() -> (tempfile::TempDir, Knowledge) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let kb = Knowledge::open(path.to_str().unwrap()).unwrap();
    (dir, kb)  // keep dir alive so file isn't deleted
}

#[test]
fn open_and_reopen_preserves_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");

    {
        let kb = Knowledge::open(path.to_str().unwrap()).unwrap();
        let rom_id = kb.register_rom(&RomInfo {
            hash: "reopen-test".into(),
            title: Some("Test ROM".into()),
            platform: "gba".into(),
            region: None,
        }).unwrap();
        kb.register_symbols(rom_id, &[
            Symbol::from_decomp(0x0800_0000, Some(4), "T", "main"),
        ]).unwrap();
    }

    // Reopen — data should persist
    {
        let kb = Knowledge::open(path.to_str().unwrap()).unwrap();
        let rec = kb.get_rom_by_hash("reopen-test").unwrap().unwrap();
        assert_eq!(rec.platform, "gba");
        let sym = kb.lookup_symbol(rec.id, 0x0800_0000).unwrap().unwrap();
        assert_eq!(sym.name, "main");
    }
}

#[test]
fn nm_output_pipeline_into_db() {
    // Simulate a full nm → parse → store → query pipeline.
    let (_dir, kb) = temp_db();

    let nm_output = concat!(
        "         U _libc_start_main\n",
        "08000000 00000004 T _start\n",
        "08000004 000000f8 T main\n",
        "080000fc 00000010 t init_data\n",
        "20000000 00000004 D gVersion\n",
        "20000004 00001000 B gHeap\n",
    );

    let symbols = parse_nm_output(nm_output);
    assert_eq!(symbols.len(), 5);  // _libc_start_main is undefined, skipped

    let rom_id = kb.register_rom(&RomInfo {
        hash: "nm-pipeline-test".into(),
        title: None,
        platform: "gba".into(),
        region: None,
    }).unwrap();

    let n = kb.register_symbols(rom_id, &symbols).unwrap();
    assert_eq!(n, 5);

    // Query back
    let main = kb.lookup_symbol(rom_id, 0x0800_0004).unwrap().unwrap();
    assert_eq!(main.name, "main");
    assert_eq!(main.size, Some(0xf8));
    assert_eq!(main.source, "decomp");

    let text_syms = kb.symbols_in_range(rom_id, 0x0800_0000, 0x0801_0000).unwrap();
    assert_eq!(text_syms.len(), 3);  // _start, main, init_data

    let data_syms = kb.symbols_in_range(rom_id, 0x2000_0000, 0x2001_0000).unwrap();
    assert_eq!(data_syms.len(), 2);  // gVersion, gHeap
}

#[test]
fn register_multiple_roms_isolated() {
    let (_dir, kb) = temp_db();

    let gba_id = kb.register_rom(&RomInfo {
        hash: "gba-rom".into(), title: None, platform: "gba".into(), region: None,
    }).unwrap();
    let ds_id = kb.register_rom(&RomInfo {
        hash: "ds-rom".into(), title: None, platform: "ds".into(), region: None,
    }).unwrap();

    kb.register_symbols(gba_id, &[
        Symbol::from_decomp(0x0800_0000, None, "T", "gba_main"),
    ]).unwrap();
    kb.register_symbols(ds_id, &[
        Symbol::from_decomp(0x0200_0000, None, "T", "ds_main"),
    ]).unwrap();

    // Each ROM only sees its own symbols
    assert!(kb.lookup_symbol(gba_id, 0x0200_0000).unwrap().is_none());
    assert!(kb.lookup_symbol(ds_id,  0x0800_0000).unwrap().is_none());

    assert!(kb.lookup_symbols_by_name(gba_id, "ds_main").unwrap().is_empty());
    assert!(kb.lookup_symbols_by_name(ds_id,  "gba_main").unwrap().is_empty());
}
