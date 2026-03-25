/// Command builders for each op type.
///
/// Each function returns a Vec<String> representing the full argv — no shell
/// interpolation, no user-controlled strings in command position.

pub fn build_cmd(target: Option<&str>, jobs: Option<u8>) -> Vec<String> {
    let mut args = vec!["make".into()];
    if let Some(j) = jobs {
        args.push(format!("-j{j}"));
    }
    if let Some(t) = target {
        args.push(t.into());
    }
    args
}

pub fn clean_cmd() -> Vec<String> {
    vec!["make".into(), "clean".into()]
}

pub fn check_cmd(tool: &str) -> Vec<String> {
    vec!["which".into(), tool.into()]
}

pub fn disassemble_cmd(file: &str, offset: Option<u32>, length: Option<u32>) -> Vec<String> {
    let mut args = vec!["objdump".into(), "-d".into()];
    if let Some(o) = offset {
        args.push(format!("--start-address=0x{o:x}"));
    }
    if let Some(l) = length {
        args.push(format!("--stop-address=0x{l:x}"));
    }
    args.push(file.into());
    args
}

pub fn hexdump_cmd(file: &str, offset: Option<u32>, length: Option<u32>) -> Vec<String> {
    let mut args = vec!["xxd".into()];
    if let Some(o) = offset {
        args.push("-s".into());
        args.push(format!("0x{o:x}"));
    }
    if let Some(l) = length {
        args.push("-l".into());
        args.push(format!("{l}"));
    }
    args.push(file.into());
    args
}

pub fn grep_cmd(pattern: &str, path: Option<&str>, recursive: bool) -> Vec<String> {
    let mut args = vec!["rg".into(), "--color=never".into()];
    // rg is recursive by default; non-recursive means max-depth 1
    if !recursive {
        args.push("--max-depth=1".into());
    }
    args.push(pattern.into());
    if let Some(p) = path {
        args.push(p.into());
    }
    args
}

pub fn list_ops() -> String {
    "build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, list_ops".into()
}

// ── Execution ──────────────────────────────────────────────────────────────────

use protocol::Response;
use std::process::Command;

pub fn run_op(id: &str, args: Vec<String>, workdir: Option<&str>) -> Response {
    let Some((cmd, rest)) = args.split_first() else {
        return Response::err(id, "empty command");
    };

    let mut command = Command::new(cmd);
    command.args(rest);
    if let Some(dir) = workdir {
        command.current_dir(dir);
    }

    match command.output() {
        Err(e) => Response::err(id, format!("failed to spawn {cmd}: {e}")),
        Ok(o) => Response::success(
            id,
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
            o.status.code().unwrap_or(-1),
        ),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_no_args() {
        assert_eq!(build_cmd(None, None), vec!["make"]);
    }

    #[test]
    fn build_target_only() {
        assert_eq!(build_cmd(Some("rom.gba"), None), vec!["make", "rom.gba"]);
    }

    #[test]
    fn build_jobs_only() {
        assert_eq!(build_cmd(None, Some(4)), vec!["make", "-j4"]);
    }

    #[test]
    fn build_all_args() {
        assert_eq!(build_cmd(Some("all"), Some(8)), vec!["make", "-j8", "all"]);
    }

    #[test]
    fn clean() {
        assert_eq!(clean_cmd(), vec!["make", "clean"]);
    }

    #[test]
    fn check_simple() {
        assert_eq!(check_cmd("git"), vec!["which", "git"]);
    }

    #[test]
    fn check_cross_compiler() {
        assert_eq!(
            check_cmd("gcc-arm-none-eabi"),
            vec!["which", "gcc-arm-none-eabi"]
        );
    }

    #[test]
    fn disassemble_no_opts() {
        assert_eq!(
            disassemble_cmd("rom.elf", None, None),
            vec!["objdump", "-d", "rom.elf"]
        );
    }

    #[test]
    fn disassemble_offset_only() {
        let args = disassemble_cmd("rom.elf", Some(0x8000), None);
        assert!(args.contains(&"--start-address=0x8000".to_string()));
        assert!(!args.iter().any(|a| a.starts_with("--stop-address")));
        assert_eq!(args.last().unwrap(), "rom.elf");
    }

    #[test]
    fn disassemble_length_only() {
        let args = disassemble_cmd("rom.elf", None, Some(0x100));
        assert!(!args.iter().any(|a| a.starts_with("--start-address")));
        assert!(args.contains(&"--stop-address=0x100".to_string()));
        assert_eq!(args.last().unwrap(), "rom.elf");
    }

    #[test]
    fn disassemble_both() {
        let args = disassemble_cmd("rom.elf", Some(0x8000), Some(0x100));
        assert!(args.contains(&"--start-address=0x8000".to_string()));
        assert!(args.contains(&"--stop-address=0x100".to_string()));
        assert_eq!(args.last().unwrap(), "rom.elf");
    }

    #[test]
    fn hexdump_no_opts() {
        assert_eq!(hexdump_cmd("rom.bin", None, None), vec!["xxd", "rom.bin"]);
    }

    #[test]
    fn hexdump_offset() {
        let args = hexdump_cmd("rom.bin", Some(0x10), None);
        assert_eq!(args, vec!["xxd", "-s", "0x10", "rom.bin"]);
    }

    #[test]
    fn hexdump_length() {
        let args = hexdump_cmd("rom.bin", None, Some(32));
        assert_eq!(args, vec!["xxd", "-l", "32", "rom.bin"]);
    }

    #[test]
    fn hexdump_both() {
        let args = hexdump_cmd("rom.bin", Some(0x10), Some(32));
        assert_eq!(args, vec!["xxd", "-s", "0x10", "-l", "32", "rom.bin"]);
    }

    #[test]
    fn grep_minimal() {
        let args = grep_cmd("main", None, true);
        assert_eq!(args, vec!["rg", "--color=never", "main"]);
    }

    #[test]
    fn grep_with_path() {
        let args = grep_cmd("TODO", Some("src/"), true);
        assert_eq!(args, vec!["rg", "--color=never", "TODO", "src/"]);
    }

    #[test]
    fn grep_non_recursive() {
        let args = grep_cmd("TODO", None, false);
        assert!(args.contains(&"--max-depth=1".to_string()));
        assert!(args.contains(&"TODO".to_string()));
    }
}
