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
use protocol::{Op, Request};

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
