use std::process::Command;

use crate::protocol::{Request, Response};

const PLATFORMS: &[(&str, &str, &str)] = &[
    // (name, image, workdir)
    ("nes",  "neshax",  "/neshax"),
    ("snes", "sneshax", "/sneshax"),
    ("gbc",  "gbchax",  "/gbchax"),
    ("gba",  "gbahax",  "/gbahax"),
    ("gen",  "genhax",  "/genhax"),
    ("ds",   "dshax",   "/dshax"),
    ("n64",  "n64hax",  "/n64hax"),
    ("ps1",  "ps1hax",  "/ps1hax"),
];

pub struct PlatformInfo {
    pub name: &'static str,
    pub image: &'static str,
    pub workdir: &'static str,
}

pub fn get_platform(name: &str) -> Option<PlatformInfo> {
    PLATFORMS.iter().find(|(n, _, _)| *n == name).map(|(name, image, workdir)| PlatformInfo {
        name,
        image,
        workdir,
    })
}

pub fn list_platforms() -> String {
    PLATFORMS
        .iter()
        .map(|(name, image, _)| {
            let built = image_exists(image);
            format!("{} {:<6}  ({})", if built { "✓" } else { "✗" }, name, image)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn image_exists(image: &str) -> bool {
    Command::new("podman")
        .args(["image", "exists", image])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Vendor mount for a platform: host platforms/<name>/vendor → <workdir>/vendor
pub fn vendor_mount(platform: &str, workdir: &str) -> String {
    let repo_root = repo_root();
    format!("{repo_root}/platforms/{platform}/vendor:{workdir}/vendor")
}

/// Start a detached container that runs `sleep infinity` and returns its container ID.
pub fn start_container(platform: &PlatformInfo) -> Result<String, String> {
    let mount = vendor_mount(platform.name, platform.workdir);
    let out = Command::new("podman")
        .args([
            "run", "-d", "--rm",
            "--userns=keep-id:uid=1001,gid=1001",
            "-v", &mount,
            platform.image,
            "sleep", "infinity",
        ])
        .output()
        .map_err(|e| format!("podman run failed: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Stop and remove a container by ID.
pub fn stop_container(container_id: &str) -> Result<(), String> {
    let out = Command::new("podman")
        .args(["rm", "-f", container_id])
        .output()
        .map_err(|e| format!("podman rm failed: {e}"))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Send a typed op to the in-container agent and return its response.
/// Uses `podman exec -i` to pipe one JSON line in, one JSON line out.
pub fn exec_op(container_id: &str, request: &Request) -> Result<Response, String> {
    let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
    let input = format!("{line}\n");

    let out = Command::new("podman")
        .args(["exec", "-i", container_id, "/usr/local/bin/shim", "agent"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("podman exec failed: {e}"))?
        .wait_with_output_and_input(input.as_bytes())
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.trim()).map_err(|e| format!("agent response parse error: {e}\nraw: {stdout}"))
}

/// Build a platform image (and romhack-base if needed).
pub fn build_platform(platform: &PlatformInfo) -> Result<String, String> {
    let repo_root = repo_root();

    if !image_exists("romhack-base") {
        let out = Command::new("podman")
            .args([
                "build",
                "-f", &format!("{repo_root}/shared/Dockerfile.base"),
                "-t", "romhack-base",
                &repo_root,
            ])
            .output()
            .map_err(|e| format!("podman build base failed: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "Failed to build romhack-base:\n{}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
    }

    let out = Command::new("podman")
        .args(["build", "-t", platform.image, &format!("{repo_root}/platforms/{}", platform.name)])
        .output()
        .map_err(|e| format!("podman build failed: {e}"))?;

    if out.status.success() {
        Ok(format!("Built {} successfully.", platform.image))
    } else {
        Err(format!("Build failed:\n{}", String::from_utf8_lossy(&out.stderr)))
    }
}

fn repo_root() -> String {
    // The shim binary lives at <repo>/shim/target/…/shim. Walk up to the repo root.
    // Fall back to CWD if we can't find it.
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            // …/shim/target/<profile>/shim → go up 4 levels to repo root
            p.ancestors().nth(4).map(|a| a.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| ".".to_owned())
}

// ── Helper trait to send stdin to a spawned child ─────────────────────────────

trait WaitWithInput {
    fn wait_with_output_and_input(self, input: &[u8]) -> std::io::Result<std::process::Output>;
}

impl WaitWithInput for std::process::Child {
    fn wait_with_output_and_input(mut self, input: &[u8]) -> std::io::Result<std::process::Output> {
        use std::io::Write;
        if let Some(mut stdin) = self.stdin.take() {
            stdin.write_all(input)?;
        }
        self.wait_with_output()
    }
}
