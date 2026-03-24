use std::process::Command;

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::protocol::{Op, Request, Response};

pub async fn run() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut lines = BufReader::new(stdin).lines();
    let mut out = stdout;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Err(e) => {
                // Can't echo back an id we couldn't parse — use "?" as sentinel
                Response::err("?", format!("parse error: {e}"))
            }
            Ok(req) => dispatch(req),
        };

        let mut serialized = serde_json::to_string(&response)?;
        serialized.push('\n');
        out.write_all(serialized.as_bytes()).await?;
        out.flush().await?;
    }

    Ok(())
}

fn dispatch(req: Request) -> Response {
    let id = req.id.clone();
    let workdir = req.workdir.clone();
    match req.op {
        Op::Build { target, jobs } => run_op(&id, build_cmd(target.as_deref(), jobs), workdir.as_deref()),
        Op::Clean => run_op(&id, clean_cmd(), workdir.as_deref()),
        Op::Check { tool } => run_op(&id, check_cmd(&tool), workdir.as_deref()),
        Op::Disassemble { file, offset, length } => {
            run_op(&id, disassemble_cmd(&file, offset, length), workdir.as_deref())
        }
        Op::HexDump { file, offset, length } => run_op(&id, hexdump_cmd(&file, offset, length), workdir.as_deref()),
        Op::Grep { pattern, path, recursive } => {
            run_op(&id, grep_cmd(&pattern, path.as_deref(), recursive.unwrap_or(true)), workdir.as_deref())
        }
        Op::GitStatus => run_op(&id, vec!["git", "status", "--short"], workdir.as_deref()),
        Op::GitDiff { file } => run_op(
            &id,
            match file.as_deref() {
                Some(f) => vec!["git", "diff", f],
                None => vec!["git", "diff"],
            },
            workdir.as_deref(),
        ),
        Op::ListOps => Response {
            id,
            ok: true,
            stdout: Some(list_ops()),
            stderr: None,
            exit_code: 0,
            error: None,
        },
    }
}

// ── Command builders ──────────────────────────────────────────────────────────
// These functions return the argv for a known, fixed command. No shell
// interpolation, no user-controlled strings in command position.

fn build_cmd(target: Option<&str>, jobs: Option<u8>) -> Vec<&str> {
    let mut args = vec!["make"];
    if let Some(j) = jobs {
        // jobs count comes from a validated u8, not a raw string
        // We use a leak here to get a &str with static-ish lifetime for the vec.
        // In practice this runs once per op invocation, so the leak is bounded.
        let j_str: &'static str = Box::leak(format!("-j{j}").into_boxed_str());
        args.push(j_str);
    }
    if let Some(t) = target {
        args.push(t);
    }
    args
}

fn clean_cmd() -> Vec<&'static str> {
    vec!["make", "clean"]
}

fn check_cmd(tool: &str) -> Vec<&str> {
    // tool name is passed as an argument to `which`, not as a command itself
    vec!["which", tool]
}

fn disassemble_cmd(file: &str, offset: Option<u32>, length: Option<u32>) -> Vec<&str> {
    let mut args = vec!["objdump", "-d"];
    if let Some(o) = offset {
        let s: &'static str = Box::leak(format!("--start-address=0x{o:x}").into_boxed_str());
        args.push(s);
    }
    if let Some(l) = length {
        let s: &'static str = Box::leak(format!("--stop-address=0x{l:x}").into_boxed_str());
        args.push(s);
    }
    args.push(file);
    args
}

fn hexdump_cmd(file: &str, offset: Option<u32>, length: Option<u32>) -> Vec<&str> {
    let mut args = vec!["xxd"];
    if let Some(o) = offset {
        let s: &'static str = Box::leak(format!("-s").into_boxed_str());
        let v: &'static str = Box::leak(format!("0x{o:x}").into_boxed_str());
        args.push(s);
        args.push(v);
    }
    if let Some(l) = length {
        let s: &'static str = Box::leak(format!("-l").into_boxed_str());
        let v: &'static str = Box::leak(format!("{l}").into_boxed_str());
        args.push(s);
        args.push(v);
    }
    args.push(file);
    args
}

fn grep_cmd<'a>(pattern: &'a str, path: Option<&'a str>, recursive: bool) -> Vec<&'a str> {
    let mut args = vec!["rg", "--color=never"];
    if recursive {
        args.push("-r");
    }
    args.push(pattern);
    if let Some(p) = path {
        args.push(p);
    }
    args
}

fn list_ops() -> String {
    "build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, list_ops".to_owned()
}

// ── Execution ─────────────────────────────────────────────────────────────────

fn run_op(id: &str, args: Vec<&str>, workdir: Option<&str>) -> Response {
    let (cmd, rest) = match args.split_first() {
        Some(pair) => pair,
        None => return Response::err(id, "empty command"),
    };

    let mut command = Command::new(cmd);
    command.args(rest);
    if let Some(dir) = workdir {
        command.current_dir(dir);
    }
    let output = command.output();

    match output {
        Err(e) => Response::err(id, format!("failed to spawn {cmd}: {e}")),
        Ok(o) => Response::success(
            id,
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
            o.status.code().unwrap_or(-1),
        ),
    }
}
