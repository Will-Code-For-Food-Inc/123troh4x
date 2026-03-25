/// Typed ops sent from onmyoji (host) to tsukumogami (in-container).
///
/// Each variant maps to a well-known operation for the platform toolchain.
/// tsukumogami never constructs shell strings from user-supplied data — it only
/// executes the pre-defined commands corresponding to each op.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    // ── Universal ops (all platforms) ────────────────────────────────────────
    Build {
        target: Option<String>,
        jobs: Option<u8>,
    },
    Clean,
    Check {
        tool: String,
    },
    Disassemble {
        file: String,
        offset: Option<u32>,
        length: Option<u32>,
    },
    HexDump {
        file: String,
        offset: Option<u32>,
        length: Option<u32>,
    },
    Grep {
        pattern: String,
        path: Option<String>,
        recursive: Option<bool>,
    },
    GitStatus,
    GitDiff {
        file: Option<String>,
    },
    /// Query what ops this agent supports.
    ListOps,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub id: String,
    /// Optional working directory inside the container. If absent, tsukumogami
    /// runs commands from its default directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    #[serde(flatten)]
    pub op: Op,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Response {
    pub id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Response {
    pub fn success(id: impl Into<String>, stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            id: id.into(),
            ok: exit_code == 0,
            stdout: Some(stdout),
            stderr: if stderr.is_empty() { None } else { Some(stderr) },
            exit_code,
            error: None,
        }
    }

    pub fn err(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ok: false,
            stdout: None,
            stderr: None,
            exit_code: -1,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Op serialization roundtrips ───────────────────────────────────────────

    fn roundtrip(op: &Op) -> Op {
        let json = serde_json::to_string(op).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    #[test]
    fn build_minimal() {
        let op = Op::Build { target: None, jobs: None };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn build_full() {
        let op = Op::Build { target: Some("rom.gba".into()), jobs: Some(4) };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn clean() {
        assert_eq!(roundtrip(&Op::Clean), Op::Clean);
    }

    #[test]
    fn check() {
        let op = Op::Check { tool: "gcc-arm-none-eabi".into() };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn disassemble_minimal() {
        let op = Op::Disassemble { file: "rom.elf".into(), offset: None, length: None };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn disassemble_full() {
        let op = Op::Disassemble {
            file: "rom.elf".into(),
            offset: Some(0x8000),
            length: Some(0x100),
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn hexdump_minimal() {
        let op = Op::HexDump { file: "rom.bin".into(), offset: None, length: None };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn hexdump_full() {
        let op = Op::HexDump {
            file: "rom.bin".into(),
            offset: Some(0x10),
            length: Some(32),
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn grep_minimal() {
        let op = Op::Grep { pattern: "main".into(), path: None, recursive: None };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn grep_full() {
        let op = Op::Grep {
            pattern: "TODO".into(),
            path: Some("src/".into()),
            recursive: Some(false),
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn git_status() {
        assert_eq!(roundtrip(&Op::GitStatus), Op::GitStatus);
    }

    #[test]
    fn git_diff_none() {
        let op = Op::GitDiff { file: None };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn git_diff_file() {
        let op = Op::GitDiff { file: Some("main.c".into()) };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn list_ops() {
        assert_eq!(roundtrip(&Op::ListOps), Op::ListOps);
    }

    // ── Request roundtrips ────────────────────────────────────────────────────

    #[test]
    fn request_with_workdir() {
        let req = Request {
            id: "abc-123".into(),
            workdir: Some("/dshax/vendor".into()),
            op: Op::GitStatus,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn request_without_workdir() {
        let req = Request {
            id: "abc-456".into(),
            workdir: None,
            op: Op::Clean,
        };
        let json = serde_json::to_string(&req).unwrap();
        // workdir should be absent from JSON when None
        assert!(!json.contains("workdir"));
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    // ── Response constructors ─────────────────────────────────────────────────

    #[test]
    fn response_success_exit_zero() {
        let r = Response::success("id1", "output".into(), "".into(), 0);
        assert!(r.ok);
        assert_eq!(r.exit_code, 0);
        assert_eq!(r.stdout.as_deref(), Some("output"));
        assert!(r.stderr.is_none()); // empty stderr omitted
        assert!(r.error.is_none());
    }

    #[test]
    fn response_success_nonzero_exit() {
        let r = Response::success("id2", "".into(), "err".into(), 1);
        assert!(!r.ok);
        assert_eq!(r.exit_code, 1);
        assert_eq!(r.stderr.as_deref(), Some("err"));
    }

    #[test]
    fn response_err() {
        let r = Response::err("id3", "spawn failed");
        assert!(!r.ok);
        assert_eq!(r.exit_code, -1);
        assert_eq!(r.error.as_deref(), Some("spawn failed"));
        assert!(r.stdout.is_none());
        assert!(r.stderr.is_none());
    }

    #[test]
    fn response_stderr_omitted_when_empty() {
        let r = Response::success("id4", "out".into(), "".into(), 0);
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains("stderr"));
    }

    #[test]
    fn response_stderr_present_when_nonempty() {
        let r = Response::success("id5", "out".into(), "warning".into(), 0);
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("stderr"));
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn unknown_op_tag_is_error() {
        let bad = r#"{"op":"teleport","destination":"mars"}"#;
        assert!(serde_json::from_str::<Op>(bad).is_err());
    }

}
