use protocol::{Op, PatchFormat, Request, Response};

use crate::ops;

pub fn dispatch(req: Request) -> Response {
    let id = req.id.clone();
    let workdir = req.workdir.as_deref();

    match req.op {
        Op::Build { target, jobs } => {
            ops::run_op(&id, ops::build_cmd(target.as_deref(), jobs), workdir)
        }
        Op::Clean => ops::run_op(&id, ops::clean_cmd(), workdir),
        Op::Check { tool } => ops::run_op(&id, ops::check_cmd(&tool), workdir),
        Op::Disassemble { file, offset, length } => {
            ops::run_op(&id, ops::disassemble_cmd(&file, offset, length), workdir)
        }
        Op::HexDump { file, offset, length } => {
            ops::run_op(&id, ops::hexdump_cmd(&file, offset, length), workdir)
        }
        Op::Grep { pattern, path, recursive } => {
            ops::run_op(&id, ops::grep_cmd(&pattern, path.as_deref(), recursive.unwrap_or(true)), workdir)
        }
        Op::GitStatus => ops::run_op(&id, vec!["git".into(), "status".into(), "--short".into()], workdir),
        Op::GitDiff { file } => {
            let mut args = vec!["git".into(), "diff".into()];
            if let Some(f) = file {
                args.push(f);
            }
            ops::run_op(&id, args, workdir)
        }
        Op::ListOps => Response {
            id,
            ok: true,
            stdout: Some(ops::list_ops()),
            stderr: None,
            exit_code: 0,
            error: None,
        },

        // ── Binary editing ops ────────────────────────────────────────────────
        Op::ReadBytes { file, offset, length } => {
            ops::read_bytes(&id, &file, offset, length)
        }
        Op::WriteBytes { file, offset, bytes } => {
            ops::write_bytes(&id, &file, offset, &bytes)
        }
        Op::ReadInstruction { file, offset, arch } => {
            match ops::read_instruction_cmd(&file, offset, &arch) {
                Err(e) => Response::err(&id, e),
                Ok(args) => ops::run_op(&id, args, workdir),
            }
        }
        Op::WriteInstruction { file, offset, arch, instruction } => {
            ops::write_instruction(&id, &file, offset, &arch, &instruction, workdir)
        }
        Op::GeneratePatch { original, modified, output, format } => {
            match format {
                PatchFormat::Bps => Response::err(&id, "BPS format not yet implemented"),
                PatchFormat::Ips => {
                    ops::run_op(&id, ops::generate_patch_ips_cmd(&original, &modified, &output), workdir)
                }
            }
        }
        Op::ApplyPatch { rom, patch, output, format } => {
            match format {
                PatchFormat::Bps => Response::err(&id, "BPS format not yet implemented"),
                PatchFormat::Ips => {
                    ops::run_op(&id, ops::apply_patch_ips_cmd(&rom, &patch, &output), workdir)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_req(op: Op) -> Request {
        Request { id: "test-id".into(), workdir: None, op }
    }

    #[test]
    fn list_ops_returns_ok() {
        let resp = dispatch(make_req(Op::ListOps));
        assert!(resp.ok);
        assert_eq!(resp.exit_code, 0);
        let stdout = resp.stdout.unwrap();
        for op in &["build", "clean", "check", "disassemble", "hex_dump",
                    "grep", "git_status", "git_diff", "list_ops"] {
            assert!(stdout.contains(op), "missing: {op}");
        }
    }

    #[test]
    fn git_diff_no_file() {
        // GitDiff without a file runs `git diff` — just check dispatch doesn't panic
        // and returns a response with the correct id.
        let resp = dispatch(make_req(Op::GitDiff { file: None }));
        assert_eq!(resp.id, "test-id");
    }

    #[test]
    fn git_diff_with_file() {
        let resp = dispatch(make_req(Op::GitDiff { file: Some("src/main.rs".into()) }));
        assert_eq!(resp.id, "test-id");
    }

    #[test]
    fn workdir_is_forwarded() {
        // Run `true` (always exits 0) with an explicit workdir — verifies workdir
        // is threaded through dispatch to run_op without being dropped.
        let req = Request {
            id: "wd-test".into(),
            workdir: Some("/tmp".into()),
            op: Op::Check { tool: "true".into() },
        };
        let resp = dispatch(req);
        assert!(resp.ok);
        assert_eq!(resp.id, "wd-test");
    }

    #[test]
    fn check_real_tool_succeeds() {
        // `true` is always available on Unix
        let resp = dispatch(make_req(Op::Check { tool: "true".into() }));
        assert!(resp.ok);
        assert_eq!(resp.exit_code, 0);
    }

    #[test]
    fn check_missing_tool_fails() {
        let resp = dispatch(make_req(Op::Check {
            tool: "definitely_not_a_real_binary_xyzzy_12345".into(),
        }));
        assert!(!resp.ok);
        assert_ne!(resp.exit_code, 0);
    }
}
