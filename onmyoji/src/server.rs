use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

use crate::{podman, sessions::{Session, SessionStore}};
use knowledge::{Annotation, Knowledge, RomInfo, hash_rom_file, parse_nm_output};
use protocol::{Op, Request, Response};

// ── Parameter types ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct PlatformParam {
    /// Platform name: nes, snes, gbc, gba, gen, ds, n64, ps1
    platform: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SessionParam {
    /// Session ID returned by start_session
    session_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RunOpParams {
    /// Session ID returned by start_session
    session_id: String,
    /// Operation: build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, list_ops
    op: String,
    /// Op-specific parameters as a JSON object (e.g. {"target":"all","jobs":4} for build)
    #[serde(default)]
    params: serde_json::Value,
    /// Working directory inside the container (e.g. "/dshax/vendor/pokeplatinum")
    workdir: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DebugPortParams {
    /// Platform name
    platform: String,
    /// Port to expose on the host (default: 2345)
    port: Option<u16>,
}

// ── Parameter types (knowledge tools) ────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HashRomParams {
    /// Absolute path to the ROM file on the host.
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RegisterRomParams {
    /// CRC32 hash returned by hash_rom (8 hex chars).
    hash: String,
    /// Platform: gba, ds, nes, snes, gbc, gen, n64, ps1
    platform: String,
    /// Human-readable title (optional).
    title: Option<String>,
    /// Region string, e.g. "US", "JP", "EU" (optional).
    region: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LookupSymbolParams {
    /// ROM id returned by register_rom.
    rom_id: i64,
    /// Virtual address (decimal or hex prefix 0x accepted).
    address: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SearchSymbolsParams {
    /// ROM id returned by register_rom.
    rom_id: i64,
    /// Substring to search for in symbol names (case-insensitive).
    pattern: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SymbolsInRangeParams {
    /// ROM id returned by register_rom.
    rom_id: i64,
    /// Start of range (inclusive), decimal or 0x hex.
    start: String,
    /// End of range (exclusive), decimal or 0x hex.
    end: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AddAnnotationParams {
    /// ROM id returned by register_rom.
    rom_id: i64,
    /// Address the annotation is attached to, decimal or 0x hex.
    address: String,
    /// Free-form description of what this address does.
    description: String,
    /// Provenance tag: "user", "decomp", "community", etc.
    #[serde(default = "default_source")]
    source: String,
}

fn default_source() -> String { "user".into() }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetAnnotationsParams {
    /// ROM id returned by register_rom.
    rom_id: i64,
    /// Address to fetch annotations for, decimal or 0x hex.
    address: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct IngestElfSymbolsParams {
    /// Session ID returned by start_session.
    session_id: String,
    /// Absolute path to the ELF file inside the container.
    elf_path: String,
    /// ROM id to store symbols under (returned by register_rom).
    rom_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CopyToSessionParams {
    /// Session ID returned by start_session.
    session_id: String,
    /// Absolute path to the source file on the host.
    host_path: String,
    /// Absolute destination path inside the container.
    container_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CopyFromSessionParams {
    /// Session ID returned by start_session.
    session_id: String,
    /// Absolute source path inside the container.
    container_path: String,
    /// Absolute destination path on the host.
    host_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListBuildTargetsParams {
    /// Session ID returned by start_session.
    session_id: String,
    /// Working directory inside the container where the Makefile lives.
    /// E.g. "/dshax/vendor/pokeplatinum"
    workdir: Option<String>,
    /// Path to the Makefile, relative to workdir. Defaults to "Makefile".
    file: Option<String>,
}

fn parse_addr(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u32>().map_err(|e| e.to_string())
    }
}

// ── Server ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct OnmyojiServer {
    root: String,
    sessions: SessionStore,
    kb: Knowledge,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl OnmyojiServer {
    pub fn new(root: String, kb: Knowledge) -> Self {
        Self {
            root,
            sessions: SessionStore::default(),
            kb,
            tool_router: Self::tool_router(),
        }
    }

    /// List all platforms and whether their container image is built.
    #[tool(description = "List all available platforms and whether their container image is built.")]
    fn list_platforms(&self) -> String {
        podman::list_platforms()
    }

    /// Get the Dockerfile for a platform.
    #[tool(description = "Get the Dockerfile for a platform.")]
    fn get_platform_info(&self, Parameters(PlatformParam { platform }): Parameters<PlatformParam>) -> String {
        match podman::get_platform(&platform) {
            None => format!("Unknown platform: {platform}. Valid: nes, snes, gbc, gba, gen, ds, n64, ps1"),
            Some(info) => {
                let root = std::path::PathBuf::from(&self.root);
                let dockerfile = std::fs::read_to_string(
                    root.join("platforms").join(info.name).join("Dockerfile"),
                )
                .unwrap_or_else(|e| format!("(could not read Dockerfile: {e})"));
                let launch = std::fs::read_to_string(
                    root.join("platforms").join(info.name).join(info.image),
                )
                .unwrap_or_else(|e| format!("(could not read launch script: {e})"));
                format!("=== Dockerfile ===\n{dockerfile}\n=== Launch script ===\n{launch}")
            }
        }
    }

    /// Build the container image for a platform. Builds romhack-base first if needed.
    #[tool(description = "Build the container image for a platform. Builds romhack-base first if needed.")]
    fn build_platform(&self, Parameters(PlatformParam { platform }): Parameters<PlatformParam>) -> String {
        match podman::get_platform(&platform) {
            None => format!("Unknown platform: {platform}"),
            Some(info) => match podman::build_platform(&self.root, &info) {
                Ok(msg) => msg,
                Err(e) => e,
            },
        }
    }

    /// Start a persistent container session for a platform. Returns a session_id.
    #[tool(
        description = "Start a persistent container session for a platform. Returns a session_id for use with run_op and stop_session."
    )]
    fn start_session(&self, Parameters(PlatformParam { platform }): Parameters<PlatformParam>) -> String {
        match podman::get_platform(&platform) {
            None => format!("Unknown platform: {platform}"),
            Some(info) => match podman::start_container(&self.root, &info) {
                Err(e) => format!("Failed to start session: {e}"),
                Ok(container_id) => {
                    let session_id = uuid::Uuid::new_v4().to_string();
                    self.sessions.insert(
                        session_id.clone(),
                        Session { container_id, platform: platform.clone() },
                    );
                    format!("session_id: {session_id}")
                }
            },
        }
    }

    /// Run a typed operation in an active session.
    #[tool(
        description = "Run a typed operation in an active container session. `op` is one of: build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, list_ops, read_bytes, write_bytes, read_instruction, write_instruction, generate_patch, apply_patch. `params` is an op-specific JSON object (e.g. {\"file\":\"/gbahax/rom.gba\",\"offset\":0x8000100,\"arch\":\"Thumb\"} for read_instruction)."
    )]
    fn run_op(&self, Parameters(RunOpParams { session_id, op, params, workdir }): Parameters<RunOpParams>) -> String {
        let session = match self.sessions.get(&session_id) {
            None => return format!("Unknown session: {session_id}"),
            Some(s) => s,
        };

        let op_value = {
            let mut v = match params {
                serde_json::Value::Object(m) => m,
                serde_json::Value::Null => serde_json::Map::new(),
                _ => return "params must be a JSON object or null".to_owned(),
            };
            v.insert("op".to_owned(), serde_json::Value::String(op));
            serde_json::Value::Object(v)
        };

        let typed_op: Op = match serde_json::from_value(op_value) {
            Ok(o) => o,
            Err(e) => return format!("Invalid op or params: {e}"),
        };

        let request = Request {
            id: uuid::Uuid::new_v4().to_string(),
            workdir,
            op: typed_op,
        };

        match podman::exec_op(&session.container_id, &request) {
            Err(e) => format!("exec failed: {e}"),
            Ok(Response { ok, stdout, stderr, exit_code, error, .. }) => {
                let mut parts = Vec::new();
                if let Some(o) = stdout { if !o.is_empty() { parts.push(o); } }
                if let Some(e) = stderr { parts.push(format!("[stderr]\n{e}")); }
                if !ok { parts.push(format!("[exit code: {exit_code}]")); }
                if let Some(e) = error { parts.push(format!("[error] {e}")); }
                if parts.is_empty() { "(no output)".to_owned() } else { parts.join("\n") }
            }
        }
    }

    /// Stop and remove a container session.
    #[tool(description = "Stop and remove a container session.")]
    fn stop_session(&self, Parameters(SessionParam { session_id }): Parameters<SessionParam>) -> String {
        match self.sessions.remove(&session_id) {
            None => format!("Unknown session: {session_id}"),
            Some(session) => match podman::stop_container(&session.container_id) {
                Ok(()) => format!("Stopped session {session_id}"),
                Err(e) => format!("Failed to stop container: {e}"),
            },
        }
    }

    // ── Knowledge tools ───────────────────────────────────────────────────────

    /// Compute the CRC32 hash of a ROM file. Returns an 8-character lowercase hex string.
    /// Use this hash to register the ROM and look up cached knowledge.
    #[tool(description = "Compute the CRC32 hash of a ROM file. Returns an 8-char lowercase hex string used as the ROM's identity key.")]
    fn hash_rom(&self, Parameters(HashRomParams { path }): Parameters<HashRomParams>) -> String {
        match hash_rom_file(&path) {
            Ok(h) => h,
            Err(e) => format!("error: {e}"),
        }
    }

    /// Register a ROM in the knowledge store. Returns the integer rom_id.
    /// If the ROM hash is already registered, returns the existing id (idempotent).
    #[tool(description = "Register a ROM in the knowledge store by hash. Returns the rom_id. Safe to call multiple times — returns the same id for the same hash.")]
    fn register_rom(
        &self,
        Parameters(RegisterRomParams { hash, platform, title, region }): Parameters<RegisterRomParams>,
    ) -> String {
        match self.kb.register_rom(&RomInfo { hash, title, platform, region }) {
            Ok(id) => id.to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Look up the symbol at an exact address. Returns JSON or "null" if none found.
    #[tool(description = "Look up the symbol at an exact virtual address in the knowledge store. Returns JSON with fields: address, size, kind, name, source.")]
    fn lookup_symbol(
        &self,
        Parameters(LookupSymbolParams { rom_id, address }): Parameters<LookupSymbolParams>,
    ) -> String {
        let addr = match parse_addr(&address) {
            Ok(a) => a,
            Err(e) => return format!("error: invalid address '{address}': {e}"),
        };
        match self.kb.lookup_symbol(rom_id, addr) {
            Err(e) => format!("error: {e}"),
            Ok(None) => "null".to_owned(),
            Ok(Some(sym)) => serde_json::json!({
                "address": sym.address,
                "size":    sym.size,
                "kind":    sym.kind,
                "name":    sym.name,
                "source":  sym.source,
            }).to_string(),
        }
    }

    /// Search symbols by name substring (case-insensitive). Returns a JSON array.
    #[tool(description = "Search symbols whose name contains the given pattern (case-insensitive substring match). Returns a JSON array of symbol objects.")]
    fn search_symbols(
        &self,
        Parameters(SearchSymbolsParams { rom_id, pattern }): Parameters<SearchSymbolsParams>,
    ) -> String {
        match self.kb.lookup_symbols_by_name(rom_id, &pattern) {
            Err(e) => format!("error: {e}"),
            Ok(syms) => {
                let arr: Vec<_> = syms.iter().map(|sym| serde_json::json!({
                    "address": sym.address,
                    "size":    sym.size,
                    "kind":    sym.kind,
                    "name":    sym.name,
                    "source":  sym.source,
                })).collect();
                serde_json::to_string(&arr).unwrap_or_else(|e| format!("error: {e}"))
            }
        }
    }

    /// Return all symbols in a virtual address range [start, end). Returns a JSON array.
    #[tool(description = "Return all symbols in the virtual address range [start, end). Returns a JSON array of symbol objects ordered by address.")]
    fn symbols_in_range(
        &self,
        Parameters(SymbolsInRangeParams { rom_id, start, end }): Parameters<SymbolsInRangeParams>,
    ) -> String {
        let s = match parse_addr(&start) {
            Ok(a) => a,
            Err(e) => return format!("error: invalid start '{start}': {e}"),
        };
        let e = match parse_addr(&end) {
            Ok(a) => a,
            Err(e) => return format!("error: invalid end '{end}': {e}"),
        };
        match self.kb.symbols_in_range(rom_id, s, e) {
            Err(err) => format!("error: {err}"),
            Ok(syms) => {
                let arr: Vec<_> = syms.iter().map(|sym| serde_json::json!({
                    "address": sym.address,
                    "size":    sym.size,
                    "kind":    sym.kind,
                    "name":    sym.name,
                    "source":  sym.source,
                })).collect();
                serde_json::to_string(&arr).unwrap_or_else(|err| format!("error: {err}"))
            }
        }
    }

    /// Add a free-form annotation to an address in the knowledge store.
    /// Returns the new annotation id.
    #[tool(description = "Add a text annotation to a virtual address in the knowledge store. Returns the annotation id.")]
    fn add_annotation(
        &self,
        Parameters(AddAnnotationParams { rom_id, address, description, source }): Parameters<AddAnnotationParams>,
    ) -> String {
        let addr = match parse_addr(&address) {
            Ok(a) => a,
            Err(e) => return format!("error: invalid address '{address}': {e}"),
        };
        let ann = Annotation { address: addr, description, validated: false, source };
        match self.kb.add_annotation(rom_id, &ann) {
            Ok(id) => id.to_string(),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Get all annotations for a virtual address. Returns a JSON array.
    #[tool(description = "Get all annotations attached to a virtual address. Returns a JSON array of annotation objects.")]
    fn get_annotations(
        &self,
        Parameters(GetAnnotationsParams { rom_id, address }): Parameters<GetAnnotationsParams>,
    ) -> String {
        let addr = match parse_addr(&address) {
            Ok(a) => a,
            Err(e) => return format!("error: invalid address '{address}': {e}"),
        };
        match self.kb.get_annotations(rom_id, addr) {
            Err(e) => format!("error: {e}"),
            Ok(anns) => {
                let arr: Vec<_> = anns.iter().map(|a| serde_json::json!({
                    "address":     a.address,
                    "description": a.description,
                    "validated":   a.validated,
                    "source":      a.source,
                })).collect();
                serde_json::to_string(&arr).unwrap_or_else(|e| format!("error: {e}"))
            }
        }
    }

    /// Run arm-none-eabi-nm on an ELF inside an active session and ingest the
    /// symbol table into the knowledge store.  Returns a summary string with
    /// the count of newly inserted symbols.
    #[tool(description = "Run arm-none-eabi-nm on an ELF file inside an active container session and store the symbol table in the knowledge DB. Returns 'inserted N symbols' (0 if already cached).")]
    fn ingest_elf_symbols(
        &self,
        Parameters(IngestElfSymbolsParams { session_id, elf_path, rom_id }): Parameters<IngestElfSymbolsParams>,
    ) -> String {
        let session = match self.sessions.get(&session_id) {
            None => return format!("error: unknown session {session_id}"),
            Some(s) => s,
        };
        let nm_out = match podman::nm_symbols(&session.container_id, &elf_path) {
            Ok(o) => o,
            Err(e) => return format!("error running nm on {elf_path}: {e}"),
        };
        let symbols = parse_nm_output(&nm_out);
        match self.kb.register_symbols(rom_id, &symbols) {
            Err(e) => format!("error storing symbols: {e}"),
            Ok(n) => format!("inserted {n} symbols ({} total in nm output)", symbols.len()),
        }
    }

    /// Copy a file from the host into an active container session.
    /// Use this to move a ROM or other asset into the container for binary editing.
    #[tool(description = "Copy a file from the host filesystem into an active container session. Use this to load a ROM into the container before running binary editing ops.")]
    fn copy_to_session(
        &self,
        Parameters(CopyToSessionParams { session_id, host_path, container_path }): Parameters<CopyToSessionParams>,
    ) -> String {
        let session = match self.sessions.get(&session_id) {
            None => return format!("error: unknown session {session_id}"),
            Some(s) => s,
        };
        match podman::copy_to_container(&session.container_id, &host_path, &container_path) {
            Ok(()) => format!("copied {host_path} → {container_path}"),
            Err(e) => format!("error: {e}"),
        }
    }

    /// Copy a file from an active container session back to the host.
    /// Use this to retrieve patch files, built binaries, or other outputs.
    #[tool(description = "Copy a file from an active container session to the host filesystem. Use this to retrieve patch files or build outputs after binary editing.")]
    fn copy_from_session(
        &self,
        Parameters(CopyFromSessionParams { session_id, container_path, host_path }): Parameters<CopyFromSessionParams>,
    ) -> String {
        let session = match self.sessions.get(&session_id) {
            None => return format!("error: unknown session {session_id}"),
            Some(s) => s,
        };
        match podman::copy_from_container(&session.container_id, &container_path, &host_path) {
            Ok(()) => format!("copied {container_path} → {host_path}"),
            Err(e) => format!("error: {e}"),
        }
    }

    /// List available Make targets in an active container session.
    /// Runs `make -pRrq` inside the container and returns a sorted, deduplicated
    /// list of non-special, non-pattern targets (one per line).
    #[tool(description = "List Make targets available in a container session's workdir. Returns one target name per line, sorted. Use before calling run_op with Build to know what targets exist.")]
    fn list_build_targets(
        &self,
        Parameters(ListBuildTargetsParams { session_id, workdir, file }): Parameters<ListBuildTargetsParams>,
    ) -> String {
        let session = match self.sessions.get(&session_id) {
            None => return format!("error: unknown session {session_id}"),
            Some(s) => s,
        };
        let op = Op::ListTargets { file };
        let request = Request {
            id: uuid::Uuid::new_v4().to_string(),
            workdir,
            op,
        };
        match podman::exec_op(&session.container_id, &request) {
            Err(e) => format!("error: {e}"),
            Ok(Response { ok, stdout, stderr, exit_code, error, .. }) => {
                if ok || exit_code == 2 {
                    // exit 2 is normal for `make -pRrq` (targets out of date)
                    stdout.unwrap_or_else(|| "(no targets found)".to_owned())
                } else {
                    let mut parts = Vec::new();
                    if let Some(o) = stdout { parts.push(o); }
                    if let Some(e) = stderr { parts.push(format!("[stderr] {e}")); }
                    if let Some(e) = error { parts.push(format!("[error] {e}")); }
                    parts.push(format!("[exit {exit_code}]"));
                    parts.join("\n")
                }
            }
        }
    }

    /// Start a container with a GDB stub port exposed to the host.
    #[tool(
        description = "Start a container with a GDB stub port exposed to the host. Connect your emulator's GDB stub to the port, then attach from inside the container."
    )]
    fn open_debug_port(&self, Parameters(DebugPortParams { platform, port }): Parameters<DebugPortParams>) -> String {
        let port = port.unwrap_or(2345);
        match podman::get_platform(&platform) {
            None => format!("Unknown platform: {platform}"),
            Some(info) => {
                let mount = podman::vendor_mount(&self.root, info.name, info.workdir);
                let out = std::process::Command::new("podman")
                    .args([
                        "run", "-d", "--rm",
                        "--userns=keep-id:uid=1001,gid=1001",
                        "-v", &mount,
                        "-p", &format!("{port}:{port}"),
                        info.image,
                        "sleep", "infinity",
                    ])
                    .output();
                match out {
                    Err(e) => format!("Failed to start debug session: {e}"),
                    Ok(o) if !o.status.success() => {
                        format!("Failed:\n{}", String::from_utf8_lossy(&o.stderr))
                    }
                    Ok(o) => {
                        let id = String::from_utf8_lossy(&o.stdout).trim().to_owned();
                        format!(
                            "Debug session started: {id}\nPort {port} exposed on host.\nStart your emulator's GDB stub on port {port}, then from inside the container:\n  ./debug-connect.sh host.gateway.internal {port}"
                        )
                    }
                }
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for OnmyojiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
    }
}
