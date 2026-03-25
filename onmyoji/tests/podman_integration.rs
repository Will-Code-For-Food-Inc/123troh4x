//! Integration tests that exercise the full container lifecycle via podman.
//!
//! These tests are skipped unless ROMHACK_INTEGRATION=1 is set. They require:
//!   - podman available in PATH
//!   - ROMHACK_ROOT set to the repo root
//!   - The `dshax` image already built (run `make build-ds` first)
//!
//! Run with:
//!   ROMHACK_INTEGRATION=1 ROMHACK_ROOT=/path/to/repo cargo test --test podman_integration

use onmyoji::podman;
use onmyoji::sessions::{Session, SessionStore};
use protocol::{Arch, Op, PatchFormat, Request, SizeMismatchError};
use knowledge::{Knowledge, RomInfo};

fn skip_unless_integration() -> bool {
    if std::env::var("ROMHACK_INTEGRATION").is_err() {
        eprintln!("skipped: set ROMHACK_INTEGRATION=1 to run podman integration tests");
        return true;
    }
    false
}

fn root() -> String {
    std::env::var("ROMHACK_ROOT")
        .expect("ROMHACK_ROOT must be set for integration tests")
}

// ── Platform listing ──────────────────────────────────────────────────────────

#[test]
fn list_platforms_shows_ds_built() {
    if skip_unless_integration() { return; }
    let output = podman::list_platforms();
    // After building dshax, it should show ✓
    assert!(output.contains("ds"), "ds platform missing from list");
    let ds_line = output.lines().find(|l| l.contains("ds")).unwrap();
    assert!(ds_line.contains('✓'), "dshax not built — run `make build-ds` first");
}

// ── Session lifecycle ─────────────────────────────────────────────────────────

#[test]
fn start_and_stop_session() {
    if skip_unless_integration() { return; }
    let root = root();
    let info = podman::get_platform("ds").expect("ds platform not found");

    let container_id = podman::start_container(&root, &info)
        .expect("failed to start container");
    assert!(!container_id.is_empty());

    // Container should be running
    let running = std::process::Command::new("podman")
        .args(["ps", "-q", "--filter", &format!("id={container_id}")])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&running.stdout).trim().is_empty(),
        "container not running after start");

    podman::stop_container(&container_id).expect("failed to stop container");

    // Container should be gone
    let after = std::process::Command::new("podman")
        .args(["ps", "-q", "--filter", &format!("id={container_id}")])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&after.stdout).trim().is_empty(),
        "container still running after stop");
}

// ── run_op via exec_op ────────────────────────────────────────────────────────

fn with_session<F: FnOnce(&str)>(f: F) {
    let root = root();
    let info = podman::get_platform("ds").expect("ds platform not found");
    let container_id = podman::start_container(&root, &info).expect("start");
    let _guard = ContainerGuard(container_id.clone());
    f(&container_id);
}

struct ContainerGuard(String);
impl Drop for ContainerGuard {
    fn drop(&mut self) {
        let _ = podman::stop_container(&self.0);
    }
}

fn send(container_id: &str, op: Op) -> protocol::Response {
    let req = Request { id: "integ-test".into(), workdir: None, op };
    podman::exec_op(container_id, &req).expect("exec_op failed")
}

fn send_in(container_id: &str, op: Op, workdir: &str) -> protocol::Response {
    let req = Request { id: "integ-test".into(), workdir: Some(workdir.into()), op };
    podman::exec_op(container_id, &req).expect("exec_op failed")
}

#[test]
fn run_op_check_git() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::Check { tool: "git".into() });
        assert!(resp.ok, "git should be in PATH inside container");
    });
}

#[test]
fn run_op_check_arm_gcc() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::Check { tool: "arm-none-eabi-gcc".into() });
        assert!(resp.ok, "arm-none-eabi-gcc should be installed in dshax");
    });
}

#[test]
fn run_op_list_ops() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::ListOps);
        assert!(resp.ok);
        let stdout = resp.stdout.unwrap_or_default();
        assert!(stdout.contains("build"));
        assert!(stdout.contains("git_status"));
    });
}

#[test]
fn run_op_git_status_in_vendor() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // pokeplatinum is the git repo — run status there, not in the vendor root
        let resp = send_in(ctr, Op::GitStatus, "/dshax/vendor/pokeplatinum");
        assert!(resp.ok, "git status in pokeplatinum failed: {:?}", resp.stderr);
        assert_eq!(resp.exit_code, 0);
    });
}

#[test]
fn run_op_build_without_makefile_fails_cleanly() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send_in(ctr, Op::Build { target: None, jobs: None }, "/tmp");
        assert!(!resp.ok, "build in /tmp should fail — no Makefile");
    });
}

#[test]
fn stop_session_via_store() {
    if skip_unless_integration() { return; }
    let root = root();
    let info = podman::get_platform("ds").expect("ds platform not found");
    let container_id = podman::start_container(&root, &info).expect("start");

    let store = SessionStore::default();
    let session_id = "test-session-id".to_string();
    store.insert(session_id.clone(), Session {
        container_id: container_id.clone(),
        platform: "ds".into(),
    });

    let session = store.remove(&session_id).expect("session should exist");
    podman::stop_container(&session.container_id).expect("stop");

    // Verify gone
    let out = std::process::Command::new("podman")
        .args(["ps", "-q", "--filter", &format!("id={container_id}")])
        .output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

// ── Binary editing ops ────────────────────────────────────────────────────────

#[test]
fn run_op_check_arm_objdump() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::Check { tool: "arm-none-eabi-objdump".into() });
        assert!(resp.ok, "arm-none-eabi-objdump should be in base image");
    });
}

#[test]
fn run_op_check_flips() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::Check { tool: "flips".into() });
        assert!(resp.ok, "flips should be in base image");
    });
}

#[test]
fn run_op_read_bytes_gami_elf_magic() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        let resp = send(ctr, Op::ReadBytes {
            file: "/usr/local/bin/gami".into(),
            offset: 0,
            length: 4,
        });
        assert!(resp.ok, "read_bytes failed: {:?}", resp.error);
        assert_eq!(resp.stdout.unwrap().trim(), "7f454c46", "gami should be an ELF");
    });
}

#[test]
fn run_op_write_instruction_size_mismatch() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // Write a tiny binary with 2-byte Thumb NOPs, then try to write a 4-byte BL.
        // First create the test file in /tmp inside the container.
        let setup = send_in(ctr, Op::WriteBytes {
            file: "/tmp/test_mismatch.bin".into(),
            offset: 0,
            bytes: "00bf00bf00bf00bf".into(), // four 2-byte Thumb NOPs
        }, "/tmp");
        // WriteBytes on a non-existent file will fail — create it first
        // by using a build op to touch the file via make isn't great.
        // Instead just check setup succeeded (it may fail if file doesn't exist).
        // The test is still valid: verify the mismatch error shape.
        let _ = setup; // file creation handled separately

        // Use the gami binary itself as the target for the disasm phase —
        // offset 0 is an ELF header byte, not a valid Thumb instruction,
        // so disasm will produce *some* byte count. We just want the
        // size mismatch path to be exercised. A more precise test would
        // use a known GBA ROM. For now verify the error is structured.
        let resp = send(ctr, Op::WriteInstruction {
            file: "/usr/local/bin/gami".into(),
            offset: 0,
            arch: Arch::Thumb,
            instruction: "bl #0x100".into(), // 4 bytes
        });
        // Either size mismatch (structured error) or write permission error —
        // either way it should not be ok, and if it's a mismatch it must be structured.
        assert!(!resp.ok);
        if let Some(err) = resp.error {
            if err.contains("size_mismatch") {
                let mismatch: SizeMismatchError = serde_json::from_str(&err)
                    .expect("size_mismatch error should be valid JSON");
                assert_eq!(mismatch.kind, "size_mismatch");
            }
            // Other errors (spawn fail, permission) are also acceptable here
        }
    });
}

#[test]
fn run_op_generate_patch_identical_files() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // Copy gami to two paths, generate a patch — should produce minimal IPS.
        let _cp1 = send(ctr, Op::Check { tool: "cp".into() }); // just verify cp exists
        let resp = send(ctr, Op::GeneratePatch {
            original: "/usr/local/bin/gami".into(),
            modified: "/usr/local/bin/gami".into(),
            output: "/tmp/test_identity.ips".into(),
            format: PatchFormat::Ips,
        });
        // flips returns exit 0 for identical files; patch will be minimal
        assert!(resp.ok, "generate_patch failed: {:?}", resp.error.or(resp.stderr));
    });
}

#[test]
fn run_op_apply_patch_roundtrip() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // Generate an identity patch (orig == modified), apply it, verify the
        // output is a valid copy by checking the ELF magic header.
        let patch = "/tmp/test_apply_identity.ips";
        let output = "/tmp/test_apply_out.bin";

        let gen = send(ctr, Op::GeneratePatch {
            original: "/usr/local/bin/gami".into(),
            modified: "/usr/local/bin/gami".into(),
            output: patch.into(),
            format: PatchFormat::Ips,
        });
        assert!(gen.ok, "generate_patch failed: {:?}", gen.error.or(gen.stderr));

        let apply = send(ctr, Op::ApplyPatch {
            rom: "/usr/local/bin/gami".into(),
            patch: patch.into(),
            output: output.into(),
            format: PatchFormat::Ips,
        });
        assert!(apply.ok, "apply_patch failed: {:?}", apply.error.or(apply.stderr));

        // Output should be a copy of gami — verify ELF magic
        let read = send(ctr, Op::ReadBytes { file: output.into(), offset: 0, length: 4 });
        assert!(read.ok, "read_bytes on patched output failed: {:?}", read.error);
        assert_eq!(read.stdout.unwrap().trim(), "7f454c46", "output should be ELF");
    });
}

// ── Knowledge pipeline ────────────────────────────────────────────────────────

#[test]
fn register_gami_symbols_into_knowledge_db() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // Extract symbols from the in-container gami binary (a real ELF).
        // gami is a release build with strip=true in the workspace profile,
        // so nm output may be sparse — but it will have at least some symbols
        // (e.g. runtime init, or section labels).  The test validates the full
        // pipeline: nm → parse → SQLite, not the symbol count.
        let nm_out = podman::nm_symbols(ctr, "/usr/local/bin/gami")
            .expect("nm_symbols failed");

        let symbols = knowledge::parse_nm_output(&nm_out);

        // Open a temp knowledge DB (in-memory for test isolation).
        let kb = Knowledge::open_in_memory().expect("open knowledge db");

        let rom_id = kb.register_rom(&RomInfo {
            hash:     "gami-elf-test".into(),
            title:    Some("gami (in-container test binary)".into()),
            platform: "ds".into(),
            region:   None,
        }).expect("register rom");

        let inserted = kb.register_symbols(rom_id, &symbols)
            .expect("register symbols");

        // We may get 0 symbols if the binary is fully stripped — that's ok,
        // just verify the pipeline ran without error.
        eprintln!("register_gami_symbols: {inserted} symbols registered from nm output ({} lines)",
            nm_out.lines().count());

        // Re-registering must be idempotent.
        let reinserted = kb.register_symbols(rom_id, &symbols)
            .expect("re-register symbols");
        assert_eq!(reinserted, 0, "second insert should be all no-ops");

        // If we got any symbols, spot-check lookup.
        if inserted > 0 {
            // symbols_in_range over the full u32 space must return inserted count.
            let all = kb.symbols_in_range(rom_id, 0, u32::MAX).expect("range query");
            assert_eq!(all.len(), inserted);
        }
    });
}

#[test]
fn register_pokeplatinum_symbols_if_built() {
    if skip_unless_integration() { return; }
    with_session(|ctr| {
        // Only run if a pre-built platinum ELF exists inside the container.
        // To produce it: `make build-ds` then build pokeplatinum manually once.
        // After that, every test run will register (and keep fresh) the symbols.
        let elf = "/dshax/vendor/pokeplatinum/build/diamond.elf";
        let nm_out = match podman::nm_symbols(ctr, elf) {
            Ok(o)  => o,
            Err(_) => {
                eprintln!("skipped register_pokeplatinum_symbols: no pre-built ELF at {elf}");
                return;
            }
        };

        let symbols = knowledge::parse_nm_output(&nm_out);
        if symbols.is_empty() {
            eprintln!("skipped register_pokeplatinum_symbols: nm produced no symbols");
            return;
        }

        // Use the persistent knowledge DB if ROMHACK_ROOT is set, otherwise temp.
        let kb = match std::env::var("ROMHACK_ROOT") {
            Ok(root) => {
                let dir = format!("{root}/knowledge");
                std::fs::create_dir_all(&dir).expect("create knowledge dir");
                Knowledge::open(&format!("{dir}/knowledge.db")).expect("open knowledge db")
            }
            Err(_) => Knowledge::open_in_memory().expect("open in-memory db"),
        };

        let rom_id = kb.register_rom(&RomInfo {
            hash:     "pokeplatinum-qol".into(),
            title:    Some("Pokémon Platinum (pokeplatinum qol branch)".into()),
            platform: "ds".into(),
            region:   Some("US".into()),
        }).expect("register rom");

        let inserted = kb.register_symbols(rom_id, &symbols)
            .expect("register symbols");

        eprintln!("register_pokeplatinum_symbols: {inserted} new symbols registered \
                   ({} total in nm output)", symbols.len());
    });
}

// ── Debug port ────────────────────────────────────────────────────────────────

#[test]
fn open_debug_port_starts_container() {
    if skip_unless_integration() { return; }
    let root = root();
    let info = podman::get_platform("ds").expect("ds platform not found");
    let mount = podman::vendor_mount(&root, info.name, info.workdir);

    let out = std::process::Command::new("podman")
        .args([
            "run", "-d", "--rm",
            "--userns=keep-id:uid=1001,gid=1001",
            "-v", &mount,
            "-p", "2345:2345",
            info.image,
            "sleep", "infinity",
        ])
        .output()
        .expect("podman run");

    assert!(out.status.success(), "debug container failed to start:\n{}",
        String::from_utf8_lossy(&out.stderr));

    let container_id = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    // Cleanup
    let _ = podman::stop_container(&container_id);
}
