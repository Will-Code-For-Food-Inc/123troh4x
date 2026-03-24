use protocol::{Op, Request, Response};

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
    fn list_ops_echoes_id() {
        let resp = dispatch(make_req(Op::ListOps));
        assert_eq!(resp.id, "test-id");
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
