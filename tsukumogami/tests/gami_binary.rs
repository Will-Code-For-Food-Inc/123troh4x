//! Integration tests that spawn the `gami` binary and communicate over stdin/stdout.

use std::io::Write;
use std::process::{Command, Stdio};

use protocol::{Op, Request, Response};

/// Path to the gami binary under test.
fn gami_bin() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    // current_exe is something like target/debug/deps/gami_binary-<hash>
    // walk up to target/<profile>/ and find gami there
    p.pop(); // deps/
    p.pop(); // debug/ or release/
    p.push("gami");
    p
}

/// Send a single request to a fresh gami process and return the response.
fn send_one(op: Op, workdir: Option<&str>) -> Response {
    let req = Request {
        id: "test-1".into(),
        workdir: workdir.map(Into::into),
        op,
    };
    let mut line = serde_json::to_string(&req).unwrap();
    line.push('\n');

    let mut child = Command::new(gami_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn gami");

    child.stdin.take().unwrap().write_all(line.as_bytes()).unwrap();

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).expect("parse response")
}

/// Send multiple requests over one stdin session, return all responses in order.
fn send_many(ops: Vec<Op>) -> Vec<Response> {
    let mut child = Command::new(gami_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn gami");

    let mut stdin = child.stdin.take().unwrap();
    for (i, op) in ops.iter().enumerate() {
        let req = Request { id: format!("req-{i}"), workdir: None, op: op.clone() };
        let mut line = serde_json::to_string(&req).unwrap();
        line.push('\n');
        stdin.write_all(line.as_bytes()).unwrap();
    }
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse response line"))
        .collect()
}

// ── ListOps ───────────────────────────────────────────────────────────────────

#[test]
fn list_ops_ok() {
    let resp = send_one(Op::ListOps, None);
    assert!(resp.ok);
    assert_eq!(resp.exit_code, 0);
    let stdout = resp.stdout.unwrap();
    for op in &["build", "clean", "check", "disassemble", "hex_dump",
                "grep", "git_status", "git_diff", "list_ops"] {
        assert!(stdout.contains(op), "missing op: {op}");
    }
}

// ── Check ─────────────────────────────────────────────────────────────────────

#[test]
fn check_git_found() {
    let resp = send_one(Op::Check { tool: "git".into() }, None);
    assert!(resp.ok, "git should be in PATH");
    assert_eq!(resp.exit_code, 0);
}

#[test]
fn check_missing_tool() {
    let resp = send_one(
        Op::Check { tool: "definitely_not_a_real_binary_xyzzy_12345".into() },
        None,
    );
    assert!(!resp.ok);
    assert_ne!(resp.exit_code, 0);
}

// ── GitStatus ─────────────────────────────────────────────────────────────────

#[test]
fn git_status_in_repo() {
    let resp = send_one(
        Op::GitStatus,
        Some(env!("CARGO_MANIFEST_DIR")),
    );
    // We're in a git repo, so it should succeed
    assert_eq!(resp.exit_code, 0);
}

#[test]
fn git_status_outside_repo() {
    let resp = send_one(Op::GitStatus, Some("/tmp"));
    // /tmp is not a git repo
    assert!(!resp.ok);
    assert_ne!(resp.exit_code, 0);
}

// ── Build ─────────────────────────────────────────────────────────────────────

#[test]
fn build_in_dir_without_makefile() {
    let resp = send_one(Op::Build { target: None, jobs: None }, Some("/tmp"));
    assert!(!resp.ok);
}

// ── HexDump ───────────────────────────────────────────────────────────────────

#[test]
fn hexdump_binary_file() {
    // Use the gami binary itself as the file to hexdump
    let file = gami_bin().to_string_lossy().into_owned();
    let resp = send_one(Op::HexDump { file, offset: Some(0), length: Some(16) }, None);
    assert!(resp.ok);
    let stdout = resp.stdout.unwrap();
    assert!(!stdout.is_empty(), "xxd should produce output");
}

// ── Grep ──────────────────────────────────────────────────────────────────────

#[test]
fn grep_pattern_found() {
    let resp = send_one(
        Op::Grep {
            pattern: "fn main".into(),
            path: Some(env!("CARGO_MANIFEST_DIR").to_string() + "/src"),
            recursive: Some(true),
        },
        None,
    );
    assert!(resp.ok);
    let stdout = resp.stdout.unwrap();
    assert!(!stdout.is_empty());
}

#[test]
fn grep_pattern_not_found() {
    let resp = send_one(
        Op::Grep {
            pattern: "xyzzy_pattern_that_definitely_does_not_exist_12345".into(),
            path: Some("/tmp".into()),
            recursive: Some(true),
        },
        None,
    );
    assert!(!resp.ok); // rg exits 1 when no match
}

// ── Protocol ──────────────────────────────────────────────────────────────────

#[test]
fn invalid_json_returns_sentinel_id() {
    let mut child = Command::new(gami_bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"this is not json\n").unwrap();
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: Response = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(resp.id, "?");
    assert!(!resp.ok);
    assert!(resp.error.as_deref().unwrap_or("").contains("parse error"));
}

#[test]
fn two_sequential_requests_correct_ids() {
    let resps = send_many(vec![Op::ListOps, Op::GitStatus]);
    assert_eq!(resps.len(), 2);
    assert_eq!(resps[0].id, "req-0");
    assert_eq!(resps[1].id, "req-1");
}
