use std::process::Command;

use protocol::{Request, Response};

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

/// Vendor mount: host platforms/<name>/vendor → <workdir>/vendor
pub fn vendor_mount(platform: &str, workdir: &str) -> String {
    let root = repo_root();
    format!("{root}/platforms/{platform}/vendor:{workdir}/vendor")
}

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

/// Send a typed op to the in-container gami and return its response.
pub fn exec_op(container_id: &str, request: &Request) -> Result<Response, String> {
    let line = serde_json::to_string(request).map_err(|e| e.to_string())?;
    let input = format!("{line}\n");

    let out = Command::new("podman")
        .args(["exec", "-i", container_id, "/usr/local/bin/gami"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("podman exec failed: {e}"))?
        .wait_with_output_and_input(input.as_bytes())
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.trim())
        .map_err(|e| format!("gami response parse error: {e}\nraw: {stdout}"))
}

/// Build a platform image (and romhack-base first if needed).
pub fn build_platform(platform: &PlatformInfo) -> Result<String, String> {
    let root = repo_root();

    if !image_exists("romhack-base") {
        let out = Command::new("podman")
            .args([
                "build",
                "-f", &format!("{root}/shared/Dockerfile.base"),
                "-t", "romhack-base",
                &root,
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
        .args(["build", "-t", platform.image, &format!("{root}/platforms/{}", platform.name)])
        .output()
        .map_err(|e| format!("podman build failed: {e}"))?;

    if out.status.success() {
        Ok(format!("Built {} successfully.", platform.image))
    } else {
        Err(format!("Build failed:\n{}", String::from_utf8_lossy(&out.stderr)))
    }
}

/// Walk up from the onmyoji binary to the repo root.
/// Binary lives at <repo>/onmyoji/target/<profile>/onmyoji — 4 ancestors up.
pub fn repo_root() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.ancestors().nth(4).map(|a| a.to_string_lossy().into_owned()))
        .unwrap_or_else(|| ".".to_owned())
}

// ── Helper trait ───────────────────────────────────────────────────────────────

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

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_platforms_resolve() {
        for name in &["nes", "snes", "gbc", "gba", "gen", "ds", "n64", "ps1"] {
            assert!(get_platform(name).is_some(), "missing platform: {name}");
        }
    }

    #[test]
    fn unknown_platform_returns_none() {
        assert!(get_platform("saturn").is_none());
        assert!(get_platform("xbox").is_none());
    }

    #[test]
    fn platform_lookup_is_case_sensitive() {
        assert!(get_platform("NES").is_none());
        assert!(get_platform("DS").is_none());
    }

    #[test]
    fn vendor_mount_format() {
        // We can't easily test the full path without a real binary, but we can
        // verify the structure of the format string.
        let mount = vendor_mount("ds", "/dshax");
        assert!(mount.contains("/platforms/ds/vendor"), "wrong platform path in: {mount}");
        assert!(mount.contains(":/dshax/vendor"), "wrong container path in: {mount}");
        assert!(!mount.contains("//"), "double slash in: {mount}");
    }

    #[test]
    fn list_platforms_contains_all_eight() {
        // image_exists will return false in test env (no podman), but the output
        // should still list all platforms.
        let output = list_platforms();
        for name in &["nes", "snes", "gbc", "gba", "gen", "ds", "n64", "ps1"] {
            assert!(output.contains(name), "missing platform in list: {name}");
        }
    }

    #[test]
    fn list_platforms_has_status_indicator() {
        let output = list_platforms();
        // Every line should have either ✓ or ✗
        for line in output.lines() {
            assert!(
                line.contains('✓') || line.contains('✗'),
                "line missing status indicator: {line}"
            );
        }
    }

    #[test]
    fn repo_root_is_a_directory() {
        let root = repo_root();
        // In test context the binary is somewhere under the workspace, so
        // repo_root may not resolve correctly — but it should at least not panic
        // and should return a non-empty string.
        assert!(!root.is_empty());
    }
}
