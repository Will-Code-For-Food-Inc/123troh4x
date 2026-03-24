/// Typed ops sent from the host shim to the in-container agent.
///
/// Each variant maps to a well-known operation for the platform toolchain.
/// The agent never constructs shell strings from user-supplied data — it only
/// executes the pre-defined commands corresponding to each op.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
    /// Query what ops this agent supports
    ListOps,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    /// Optional working directory inside the container. If absent, the agent
    /// runs commands from its default directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    #[serde(flatten)]
    pub op: Op,
}

#[derive(Debug, Serialize, Deserialize)]
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
