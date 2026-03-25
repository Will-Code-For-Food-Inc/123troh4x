/// Command builders for each op type.
///
/// Each function returns a Vec<String> representing the full argv — no shell
/// interpolation, no user-controlled strings in command position.

use protocol::{Arch, Response, SizeMismatchError};

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

/// List Makefile targets by running `make -pRrq` and parsing the output.
/// Returns a newline-separated list of target names, one per line.
/// Filters out implicit/special targets (those starting with `.` or containing `%`).
///
/// `workdir` sets the working directory for the make invocation.
/// `file` is the path to the Makefile, relative to workdir (default: "Makefile").
pub fn list_targets(id: &str, file: &str, workdir: Option<&str>) -> Response {
    use std::process::Command;

    // `make -pRrq` dumps the internal database without running anything.
    // Exit code 2 means "targets out of date" — normal for this invocation.
    let mut cmd = Command::new("make");
    cmd.args(["-pRrq", "-f", file]);
    if let Some(wd) = workdir {
        cmd.current_dir(wd);
    }

    let output = match cmd.output() {
        Err(e) => {
            return Response {
                id: id.to_owned(),
                ok: false,
                stdout: None,
                stderr: Some(format!("make not found or failed to spawn: {e}")),
                exit_code: -1,
                error: Some(e.to_string()),
            };
        }
        Ok(o) => o,
    };

    let exit_code = output.status.code().unwrap_or(-1);
    let targets = parse_make_targets(&String::from_utf8_lossy(&output.stdout));
    let stdout = targets.join("\n");

    Response {
        id: id.to_owned(),
        ok: true,
        stdout: if stdout.is_empty() { None } else { Some(stdout) },
        stderr: if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).into_owned())
        },
        exit_code,
        error: None,
    }
}

/// Parse the stdout of `make -pRrq` and return deduplicated, sorted target names.
fn parse_make_targets(output: &str) -> Vec<String> {
    let mut targets: Vec<&str> = output
        .lines()
        .filter_map(|line| {
            if line.starts_with(|c: char| c.is_whitespace() || c == '#') {
                return None;
            }
            let colon = line.find(':')?;
            // Skip variable assignments (`:=`, `::=`, `?=`)
            if matches!(line[colon..].as_bytes().get(1).copied(), Some(b'=')) {
                return None;
            }
            let name = line[..colon].trim();
            if name.is_empty() || name.starts_with('.') || name.contains('%') {
                return None;
            }
            Some(name)
        })
        .collect();
    targets.sort_unstable();
    targets.dedup();
    targets.iter().map(|s| s.to_string()).collect()
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
    "build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, \
     read_instruction, write_instruction, read_bytes, write_bytes, \
     generate_patch, apply_patch, list_targets, list_ops"
        .into()
}

// ── Binary editing — native Rust file I/O ─────────────────────────────────────

use std::io::{Read, Seek, SeekFrom, Write as IoWrite};

/// Read `length` bytes at `offset` from `file`, return as lowercase hex string.
pub fn read_bytes(id: &str, file: &str, offset: u32, length: u32) -> Response {
    let result = (|| -> std::io::Result<String> {
        let mut f = std::fs::File::open(file)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        let mut buf = vec![0u8; length as usize];
        f.read_exact(&mut buf)?;
        Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
    })();

    match result {
        Ok(hex) => Response {
            id: id.into(),
            ok: true,
            stdout: Some(hex),
            stderr: None,
            exit_code: 0,
            error: None,
        },
        Err(e) => Response::err(id, format!("read_bytes failed: {e}")),
    }
}

/// Write bytes (hex string) at `offset` into `file`.
pub fn write_bytes(id: &str, file: &str, offset: u32, hex: &str) -> Response {
    let bytes = match decode_hex(hex) {
        Ok(b) => b,
        Err(e) => return Response::err(id, format!("invalid hex string: {e}")),
    };

    let result = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new().write(true).open(file)?;
        f.seek(SeekFrom::Start(offset as u64))?;
        f.write_all(&bytes)
    })();

    match result {
        Ok(()) => Response {
            id: id.into(),
            ok: true,
            stdout: None,
            stderr: None,
            exit_code: 0,
            error: None,
        },
        Err(e) => Response::err(id, format!("write_bytes failed: {e}")),
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err("odd number of hex digits".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// ── Instruction ops — toolchain-backed ───────────────────────────────────────

/// Build the objdump command to disassemble one instruction at `offset`.
pub fn read_instruction_cmd(file: &str, offset: u32, arch: &Arch) -> Result<Vec<String>, String> {
    match arch {
        Arch::Thumb => Ok(vec![
            "arm-none-eabi-objdump".into(),
            "-b".into(), "binary".into(),
            "-m".into(), "arm".into(),
            "--disassembler-options=force-thumb".into(),
            "-D".into(),
            format!("--start-address=0x{offset:x}"),
            format!("--stop-address=0x{:x}", offset + 4), // max Thumb2 width
            file.into(),
        ]),
        _ => Err(format!("arch {arch:?} not yet implemented")),
    }
}

/// Assemble one instruction string to a raw binary via arm-none-eabi-as + objcopy.
/// Reads instruction from stdin (pass as `\t<instruction>\n`).
/// Returns (as_args, objcopy_args) — caller runs them in sequence.
pub fn assemble_thumb_cmds(obj_path: &str, bin_path: &str) -> (Vec<String>, Vec<String>) {
    let as_args = vec![
        "arm-none-eabi-as".into(),
        "-mthumb".into(),
        "-o".into(), obj_path.into(),
        "-".into(), // read from stdin
    ];
    let objcopy_args = vec![
        "arm-none-eabi-objcopy".into(),
        "-O".into(), "binary".into(),
        "--only-section=.text".into(),
        obj_path.into(),
        bin_path.into(),
    ];
    (as_args, objcopy_args)
}

/// Multi-step WriteInstruction:
/// 1. Disassemble at offset to learn existing byte count.
/// 2. Assemble new instruction string via stdin.
/// 3. Extract raw bytes with objcopy.
/// 4. Validate sizes match.
/// 5. Write bytes at offset via write_bytes.
pub fn write_instruction(
    id: &str,
    file: &str,
    offset: u32,
    arch: &Arch,
    instruction: &str,
    workdir: Option<&str>,
) -> Response {
    if !matches!(arch, Arch::Thumb) {
        return Response::err(id, format!("arch {arch:?} not yet implemented"));
    }

    // Phase 1: disassemble existing instruction to get byte count.
    let disasm_args = match read_instruction_cmd(file, offset, arch) {
        Ok(a) => a,
        Err(e) => return Response::err(id, e),
    };
    let disasm = run_op(id, disasm_args, workdir);
    if !disasm.ok {
        return disasm;
    }
    let existing_bytes = match parse_objdump_byte_count(disasm.stdout.as_deref().unwrap_or("")) {
        Some(n) => n,
        None => return Response::err(id, "failed to parse objdump output for existing instruction"),
    };

    // Phase 2: assemble new instruction, write obj + bin to temp paths.
    let obj_path = format!("/tmp/gami_{id}.o");
    let bin_path = format!("/tmp/gami_{id}.bin");
    let (as_args, objcopy_args) = assemble_thumb_cmds(&obj_path, &bin_path);

    let asm_input = format!("\t{instruction}\n");
    let as_resp = run_op_with_stdin(id, as_args, workdir, asm_input.as_bytes());
    if !as_resp.ok {
        return as_resp;
    }
    let objcopy_resp = run_op(id, objcopy_args, workdir);
    if !objcopy_resp.ok {
        return objcopy_resp;
    }

    // Phase 3: read assembled bytes to measure new size.
    let new_bytes = match std::fs::metadata(&bin_path) {
        Ok(m) => m.len() as u32,
        Err(e) => return Response::err(id, format!("failed to stat assembled binary: {e}")),
    };

    // Phase 4: size check.
    if new_bytes != existing_bytes {
        let mismatch = SizeMismatchError::new(existing_bytes, new_bytes);
        let _ = std::fs::remove_file(&obj_path);
        let _ = std::fs::remove_file(&bin_path);
        return Response {
            id: id.into(),
            ok: false,
            stdout: None,
            stderr: None,
            exit_code: -2,
            error: Some(serde_json::to_string(&mismatch).unwrap_or_default()),
        };
    }

    // Phase 5: read the assembled bytes and write them into the ROM.
    let assembled_hex = match std::fs::read(&bin_path) {
        Ok(b) => b.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        Err(e) => return Response::err(id, format!("failed to read assembled binary: {e}")),
    };
    let _ = std::fs::remove_file(&obj_path);
    let _ = std::fs::remove_file(&bin_path);

    write_bytes(id, file, offset, &assembled_hex)
}

/// Parse objdump -D output to extract byte count of the first instruction.
/// Sample line: "   8000100:\t01 d0       \tbne.n\t0x8000104"
/// Returns the number of bytes in the hex column (space-separated pairs).
fn parse_objdump_byte_count(output: &str) -> Option<u32> {
    for line in output.lines() {
        // objdump format: "   addr:\thex bytes\tmnemonic"
        // split_once(':') gives after_addr.1 = "\thex bytes\tmnemonic"
        let Some(after_addr) = line.split_once(':') else { continue };
        // cols[0] is empty (before first tab), cols[1] is hex bytes, cols[2] is mnemonic
        let cols: Vec<&str> = after_addr.1.splitn(3, '\t').collect();
        if cols.len() < 3 { continue; }
        let hex_col = cols[1].trim();
        if hex_col.is_empty() { continue; }
        let count = hex_col.split_whitespace().count() as u32;
        if count > 0 { return Some(count); }
    }
    None
}

// ── Patch ops — flips ─────────────────────────────────────────────────────────

pub fn generate_patch_ips_cmd(original: &str, modified: &str, output: &str) -> Vec<String> {
    vec![
        "flips".into(),
        "--create".into(), "--ips".into(),
        original.into(), modified.into(), output.into(),
    ]
}

pub fn apply_patch_ips_cmd(rom: &str, patch: &str, output: &str) -> Vec<String> {
    vec![
        "flips".into(),
        "--apply".into(), "--ips".into(),
        patch.into(), rom.into(), output.into(),
    ]
}

// ── Execution ──────────────────────────────────────────────────────────────────

use std::process::{Command, Stdio};

pub fn run_op(id: &str, args: Vec<String>, workdir: Option<&str>) -> Response {
    run_op_with_stdin(id, args, workdir, &[])
}

pub fn run_op_with_stdin(id: &str, args: Vec<String>, workdir: Option<&str>, stdin: &[u8]) -> Response {
    let Some((cmd, rest)) = args.split_first() else {
        return Response::err(id, "empty command");
    };

    let stdin_mode = if stdin.is_empty() { Stdio::null() } else { Stdio::piped() };
    let mut command = Command::new(cmd);
    command.args(rest).stdin(stdin_mode);
    if let Some(dir) = workdir {
        command.current_dir(dir);
    }

    let mut child = match command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Err(e) => return Response::err(id, format!("failed to spawn {cmd}: {e}")),
        Ok(c) => c,
    };

    if !stdin.is_empty() {
        if let Some(mut s) = child.stdin.take() {
            let _ = s.write_all(stdin);
        }
    }

    match child.wait_with_output() {
        Err(e) => Response::err(id, format!("failed to wait for {cmd}: {e}")),
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

    // ── read_instruction_cmd ──────────────────────────────────────────────────

    #[test]
    fn read_instruction_thumb_argv() {
        let args = read_instruction_cmd("rom.gba", 0x800_0100, &Arch::Thumb).unwrap();
        assert_eq!(args[0], "arm-none-eabi-objdump");
        assert!(args.contains(&"-b".to_string()));
        assert!(args.contains(&"binary".to_string()));
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"arm".to_string()));
        assert!(args.iter().any(|a| a.contains("force-thumb")));
        assert!(args.contains(&"-D".to_string()));
        assert!(args.iter().any(|a| a.contains("start-address")));
        assert!(args.iter().any(|a| a.contains("stop-address")));
        assert_eq!(args.last().unwrap(), "rom.gba");
    }

    #[test]
    fn read_instruction_unsupported_returns_err() {
        let result = read_instruction_cmd("rom.bin", 0, &Arch::Mips32);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not yet implemented"));
    }

    // ── assemble_thumb_cmds ───────────────────────────────────────────────────

    #[test]
    fn assemble_thumb_as_argv() {
        let (as_args, _) = assemble_thumb_cmds("/tmp/out.o", "/tmp/out.bin");
        assert_eq!(as_args[0], "arm-none-eabi-as");
        assert!(as_args.contains(&"-mthumb".to_string()));
        assert!(as_args.contains(&"-o".to_string()));
        assert!(as_args.contains(&"/tmp/out.o".to_string()));
        assert_eq!(as_args.last().unwrap(), "-"); // stdin
    }

    #[test]
    fn assemble_thumb_objcopy_argv() {
        let (_, objcopy_args) = assemble_thumb_cmds("/tmp/out.o", "/tmp/out.bin");
        assert_eq!(objcopy_args[0], "arm-none-eabi-objcopy");
        assert!(objcopy_args.contains(&"-O".to_string()));
        assert!(objcopy_args.contains(&"binary".to_string()));
        assert!(objcopy_args.iter().any(|a| a.contains("text")));
        assert!(objcopy_args.contains(&"/tmp/out.o".to_string()));
        assert_eq!(objcopy_args.last().unwrap(), "/tmp/out.bin");
    }

    // ── generate_patch_ips_cmd / apply_patch_ips_cmd ─────────────────────────

    #[test]
    fn generate_patch_ips_argv() {
        let args = generate_patch_ips_cmd("orig.gba", "mod.gba", "patch.ips");
        assert_eq!(args[0], "flips");
        assert!(args.contains(&"--create".to_string()));
        assert!(args.contains(&"--ips".to_string()));
        assert!(args.contains(&"orig.gba".to_string()));
        assert!(args.contains(&"mod.gba".to_string()));
        assert_eq!(args.last().unwrap(), "patch.ips");
    }

    #[test]
    fn apply_patch_ips_argv() {
        let args = apply_patch_ips_cmd("orig.gba", "patch.ips", "out.gba");
        assert_eq!(args[0], "flips");
        assert!(args.contains(&"--apply".to_string()));
        assert!(args.contains(&"--ips".to_string()));
        assert!(args.contains(&"patch.ips".to_string()));
        assert!(args.contains(&"orig.gba".to_string()));
        assert_eq!(args.last().unwrap(), "out.gba");
    }

    // ── parse_objdump_byte_count ──────────────────────────────────────────────

    #[test]
    fn parse_objdump_two_byte_thumb() {
        let output = "   8000100:\t01 d0       \tbne.n\t0x8000104\n";
        assert_eq!(parse_objdump_byte_count(output), Some(2));
    }

    #[test]
    fn parse_objdump_four_byte_thumb2() {
        let output = "   8000100:\tf0 b5 03 af \tpush\t{r4, r5, r6, r7, lr}\n";
        assert_eq!(parse_objdump_byte_count(output), Some(4));
    }

    #[test]
    fn parse_objdump_skips_header_lines() {
        let output = "\nrom.bin:     file format binary\n\n\
                      Disassembly of section .data:\n\n\
                      00000000 <.data>:\n\
                      \t0:\te0 12 ff ff \tldr\tr1, [r0, r2]\n";
        assert_eq!(parse_objdump_byte_count(output), Some(4));
    }

    #[test]
    fn parse_objdump_empty_returns_none() {
        assert_eq!(parse_objdump_byte_count(""), None);
        assert_eq!(parse_objdump_byte_count("no instructions here"), None);
    }

    // ── parse_make_targets ────────────────────────────────────────────────────

    #[test]
    fn parse_make_targets_basic() {
        let output = "all: foo bar\nfoo: src/main.c\nbar:\n";
        let targets = parse_make_targets(output);
        assert!(targets.contains(&"all".to_string()));
        assert!(targets.contains(&"foo".to_string()));
        assert!(targets.contains(&"bar".to_string()));
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn parse_make_targets_filters_dot_targets() {
        let output = ".PHONY: all\n.DEFAULT_GOAL := all\nall:\n";
        let targets = parse_make_targets(output);
        assert!(!targets.contains(&".PHONY".to_string()));
        assert!(!targets.contains(&".DEFAULT_GOAL".to_string()));
        assert!(targets.contains(&"all".to_string()));
    }

    #[test]
    fn parse_make_targets_filters_pattern_rules() {
        let output = "%.o: %.c\nall:\n";
        let targets = parse_make_targets(output);
        assert!(!targets.iter().any(|t| t.contains('%')));
        assert!(targets.contains(&"all".to_string()));
    }

    #[test]
    fn parse_make_targets_filters_variable_assignments() {
        let output = "CC := gcc\nLD = ld\nall:\n";
        let targets = parse_make_targets(output);
        assert!(!targets.contains(&"CC".to_string()));
        assert!(!targets.contains(&"LD".to_string()));
        assert!(targets.contains(&"all".to_string()));
    }

    #[test]
    fn parse_make_targets_skips_comment_lines() {
        let output = "# this is a comment\nall:\n# another comment\n";
        let targets = parse_make_targets(output);
        assert_eq!(targets, vec!["all".to_string()]);
    }

    #[test]
    fn parse_make_targets_deduplicates() {
        let output = "all: foo\nall: bar\n";
        let targets = parse_make_targets(output);
        assert_eq!(targets.iter().filter(|t| *t == "all").count(), 1);
    }

    #[test]
    fn parse_make_targets_sorted() {
        let output = "zzz:\naaa:\nmmm:\n";
        let targets = parse_make_targets(output);
        let mut sorted = targets.clone();
        sorted.sort();
        assert_eq!(targets, sorted);
    }

    #[test]
    fn parse_make_targets_empty_input() {
        assert!(parse_make_targets("").is_empty());
    }
}
