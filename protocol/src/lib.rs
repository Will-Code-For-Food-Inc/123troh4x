/// Typed ops sent from onmyoji (host) to tsukumogami (in-container).
///
/// Each variant maps to a well-known operation for the platform toolchain.
/// tsukumogami never constructs shell strings from user-supplied data — it only
/// executes the pre-defined commands corresponding to each op.
use serde::{Deserialize, Serialize};

/// CPU architecture — determines instruction encoding for read/write ops.
/// Variants beyond Thumb are accepted by the protocol but return an error
/// from tsukumogami until their toolchain support is implemented.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    Thumb,
    Arm32,
    Mips32,
    Mos6502,
    Snes65816,
}

/// Patch file format for GeneratePatch / ApplyPatch.
/// Bps is accepted by the protocol but returns an error until implemented.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatchFormat {
    Ips,
    Bps,
}

/// Structured error returned in Response::error when WriteInstruction
/// cannot proceed because the new instruction encodes to a different
/// byte length than the instruction currently at that offset.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SizeMismatchError {
    pub kind: String, // always "size_mismatch"
    pub existing_bytes: u32,
    pub new_bytes: u32,
}

impl SizeMismatchError {
    pub fn new(existing_bytes: u32, new_bytes: u32) -> Self {
        Self { kind: "size_mismatch".into(), existing_bytes, new_bytes }
    }
}

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
    /// List available Make targets in the current (or given) Makefile.
    ListTargets {
        /// Path to the Makefile. Defaults to "Makefile" in the workdir.
        file: Option<String>,
    },
    /// Query what ops this agent supports.
    ListOps,

    // ── Binary editing ops ────────────────────────────────────────────────────
    /// Decode the instruction at offset and return its mnemonic + byte length.
    ReadInstruction {
        file: String,
        offset: u32,
        arch: Arch,
    },
    /// Encode instruction string, validate size matches existing instruction,
    /// then write bytes at offset. Returns SizeMismatchError if sizes differ.
    WriteInstruction {
        file: String,
        offset: u32,
        arch: Arch,
        instruction: String,
    },
    /// Read raw bytes at offset, returned as a lowercase hex string.
    ReadBytes {
        file: String,
        offset: u32,
        length: u32,
    },
    /// Write raw bytes at offset. bytes is a lowercase hex string (e.g. "deadbeef").
    WriteBytes {
        file: String,
        offset: u32,
        bytes: String,
    },
    /// Diff two ROM files and emit a patch file.
    GeneratePatch {
        original: String,
        modified: String,
        output: String,
        format: PatchFormat,
    },
    /// Apply a patch file to a ROM. Writes output to a separate file.
    ApplyPatch {
        rom: String,
        patch: String,
        output: String,
        format: PatchFormat,
    },
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

    // ── Arch + PatchFormat roundtrips ─────────────────────────────────────────

    #[test]
    fn arch_all_variants_roundtrip() {
        let archs = [Arch::Thumb, Arch::Arm32, Arch::Mips32, Arch::Mos6502, Arch::Snes65816];
        for arch in &archs {
            let json = serde_json::to_string(arch).unwrap();
            let back: Arch = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *arch);
        }
    }

    #[test]
    fn arch_thumb_serializes_to_snake_case() {
        let json = serde_json::to_string(&Arch::Thumb).unwrap();
        assert_eq!(json, r#""thumb""#);
    }

    #[test]
    fn arch_unknown_tag_is_error() {
        assert!(serde_json::from_str::<Arch>(r#""z80""#).is_err());
    }

    #[test]
    fn patchformat_roundtrip() {
        assert_eq!(serde_json::to_string(&PatchFormat::Ips).unwrap(), r#""ips""#);
        assert_eq!(serde_json::to_string(&PatchFormat::Bps).unwrap(), r#""bps""#);
    }

    // ── Binary editing op roundtrips ──────────────────────────────────────────

    #[test]
    fn read_instruction_roundtrip() {
        let op = Op::ReadInstruction {
            file: "rom.gba".into(),
            offset: 0x8000100,
            arch: Arch::Thumb,
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn write_instruction_roundtrip() {
        let op = Op::WriteInstruction {
            file: "rom.gba".into(),
            offset: 0x8000100,
            arch: Arch::Thumb,
            instruction: "BEQ 0x08000200".into(),
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn read_bytes_roundtrip() {
        let op = Op::ReadBytes { file: "rom.gba".into(), offset: 0x100, length: 16 };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn write_bytes_roundtrip() {
        let op = Op::WriteBytes {
            file: "rom.gba".into(),
            offset: 0x100,
            bytes: "deadbeef".into(),
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn generate_patch_roundtrip() {
        let op = Op::GeneratePatch {
            original: "rom.gba".into(),
            modified: "rom_modified.gba".into(),
            output: "hack.ips".into(),
            format: PatchFormat::Ips,
        };
        assert_eq!(roundtrip(&op), op);
    }

    #[test]
    fn apply_patch_roundtrip() {
        let op = Op::ApplyPatch {
            rom: "rom.gba".into(),
            patch: "hack.ips".into(),
            output: "rom_patched.gba".into(),
            format: PatchFormat::Ips,
        };
        assert_eq!(roundtrip(&op), op);
    }

    // ── SizeMismatchError ─────────────────────────────────────────────────────

    #[test]
    fn size_mismatch_error_shape() {
        let e = SizeMismatchError::new(2, 4);
        assert_eq!(e.kind, "size_mismatch");
        assert_eq!(e.existing_bytes, 2);
        assert_eq!(e.new_bytes, 4);
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"kind\":\"size_mismatch\""));
        assert!(json.contains("\"existing_bytes\":2"));
        assert!(json.contains("\"new_bytes\":4"));
    }

}
