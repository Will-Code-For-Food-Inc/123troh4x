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
pub fn vendor_mount(root: &str, platform: &str, workdir: &str) -> String {
    format!("{root}/platforms/{platform}/vendor:{workdir}/vendor")
}

pub fn start_container(root: &str, platform: &PlatformInfo) -> Result<String, String> {
    let mount = vendor_mount(root, platform.name, platform.workdir);
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

/// Run `arm-none-eabi-nm --print-size --numeric-sort` on an ELF inside a
/// running container and return the raw stdout.
///
/// Parse the result with `knowledge::parse_nm_output`.
pub fn nm_symbols(container_id: &str, elf: &str) -> Result<String, String> {
    let out = Command::new("podman")
        .args(["exec", container_id,
               "arm-none-eabi-nm", "--print-size", "--numeric-sort", elf])
        .output()
        .map_err(|e| format!("podman exec nm failed: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Copy a file from the host into a running container.
/// `host_path` is an absolute path on the host.
/// `container_path` is an absolute path inside the container.
pub fn copy_to_container(container_id: &str, host_path: &str, container_path: &str) -> Result<(), String> {
    let out = Command::new("podman")
        .args(["cp", host_path, &format!("{container_id}:{container_path}")])
        .output()
        .map_err(|e| format!("podman cp failed: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Copy a file from a running container to the host.
/// `container_path` is an absolute path inside the container.
/// `host_path` is an absolute path on the host.
pub fn copy_from_container(container_id: &str, container_path: &str, host_path: &str) -> Result<(), String> {
    let out = Command::new("podman")
        .args(["cp", &format!("{container_id}:{container_path}"), host_path])
        .output()
        .map_err(|e| format!("podman cp failed: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Build a platform image (and romhack-base first if needed).
pub fn build_platform(root: &str, platform: &PlatformInfo) -> Result<String, String> {
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
        let cases = [
            ("nes",  "neshax",  "/neshax"),
            ("snes", "sneshax", "/sneshax"),
            ("gbc",  "gbchax",  "/gbchax"),
            ("gba",  "gbahax",  "/gbahax"),
            ("gen",  "genhax",  "/genhax"),
            ("ds",   "dshax",   "/dshax"),
            ("n64",  "n64hax",  "/n64hax"),
            ("ps1",  "ps1hax",  "/ps1hax"),
        ];
        for (name, image, workdir) in &cases {
            let p = get_platform(name).unwrap_or_else(|| panic!("missing platform: {name}"));
            assert_eq!(p.name, *name);
            assert_eq!(p.image, *image);
            assert_eq!(p.workdir, *workdir);
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
        let mount = vendor_mount("/repos/romhack", "ds", "/dshax");
        assert_eq!(mount, "/repos/romhack/platforms/ds/vendor:/dshax/vendor");
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

}
