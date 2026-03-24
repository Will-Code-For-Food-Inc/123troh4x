use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

use crate::{podman, sessions::{Session, SessionStore}};
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

// ── Server ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct OnmyojiServer {
    root: String,
    sessions: SessionStore,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl OnmyojiServer {
    pub fn new(root: String) -> Self {
        Self {
            root,
            sessions: SessionStore::default(),
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
        description = "Run a typed operation in an active container session. `op` is one of: build, clean, check, disassemble, hex_dump, grep, git_status, git_diff, list_ops. `params` is an op-specific JSON object."
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
