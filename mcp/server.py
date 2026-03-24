"""
123troh4x MCP server

Exposes the platform containers as structured execution environments for AI
agents. The agent stays outside the container; this server is the bridge.
"""

import subprocess
import json
from pathlib import Path

import mcp.server.stdio
import mcp.types as types
from mcp.server import Server

# Repo root is one level up from mcp/
REPO_ROOT = Path(__file__).parent.parent

PLATFORMS = {
    "nes":  {"image": "neshax",  "user": "neshax",  "workdir": "/neshax"},
    "snes": {"image": "sneshax", "user": "sneshax",  "workdir": "/sneshax"},
    "gbc":  {"image": "gbchax",  "user": "gbchax",   "workdir": "/gbchax"},
    "gba":  {"image": "gbahax",  "user": "gbahax",   "workdir": "/gbahax"},
    "gen":  {"image": "genhax",  "user": "genhax",   "workdir": "/genhax"},
    "ds":   {"image": "dshax",   "user": "dshax",    "workdir": "/dshax"},
    "n64":  {"image": "n64hax",  "user": "n64hax",   "workdir": "/n64hax"},
    "ps1":  {"image": "ps1hax",  "user": "ps1hax",   "workdir": "/ps1hax"},
}

server = Server("123troh4x")


def _platform_dir(platform: str) -> Path:
    names = {"nes": "nes", "snes": "snes", "gbc": "gbc", "gba": "gba",
             "gen": "gen", "ds": "ds", "n64": "n64", "ps1": "ps1"}
    return REPO_ROOT / "platforms" / names[platform]


def _image_exists(image: str) -> bool:
    result = subprocess.run(
        ["podman", "image", "exists", image],
        capture_output=True,
    )
    return result.returncode == 0


def _base_run_args(platform: str) -> list[str]:
    """Common podman run args for a platform."""
    p = PLATFORMS[platform]
    platform_dir = _platform_dir(platform)
    vendor_dir = platform_dir / "vendor"
    container_vendor = f"{p['workdir']}/vendor"
    return [
        "podman", "run", "--rm",
        "--userns=keep-id:uid=1001,gid=1001",
        "-v", f"{vendor_dir}:{container_vendor}",
        p["image"],
    ]


@server.list_tools()
async def list_tools() -> list[types.Tool]:
    return [
        types.Tool(
            name="list_platforms",
            description="List all available platforms and whether their container image is built.",
            inputSchema={"type": "object", "properties": {}, "required": []},
        ),
        types.Tool(
            name="get_platform_info",
            description="Get toolchain details, Dockerfile contents, and launch script for a platform.",
            inputSchema={
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "description": "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)",
                    }
                },
                "required": ["platform"],
            },
        ),
        types.Tool(
            name="build_platform",
            description="Build the container image for a platform. Builds romhack-base first if needed.",
            inputSchema={
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "description": "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)",
                    }
                },
                "required": ["platform"],
            },
        ),
        types.Tool(
            name="run_command",
            description=(
                "Run a one-shot command inside a platform container and return its output. "
                "The container is removed after the command completes. "
                "The vendor/ directory is mounted read-write at the platform workdir."
            ),
            inputSchema={
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "description": "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)",
                    },
                    "command": {
                        "type": "string",
                        "description": "Shell command to run inside the container",
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 60)",
                        "default": 60,
                    },
                },
                "required": ["platform", "command"],
            },
        ),
        types.Tool(
            name="start_session",
            description=(
                "Start a persistent (detached) container session for a platform. "
                "Returns a container ID that can be used with run_in_session. "
                "Call stop_session when done."
            ),
            inputSchema={
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "description": "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)",
                    }
                },
                "required": ["platform"],
            },
        ),
        types.Tool(
            name="run_in_session",
            description="Run a command in an existing persistent container session.",
            inputSchema={
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "Container ID returned by start_session",
                    },
                    "command": {
                        "type": "string",
                        "description": "Shell command to run",
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 60)",
                        "default": 60,
                    },
                },
                "required": ["container_id", "command"],
            },
        ),
        types.Tool(
            name="stop_session",
            description="Stop and remove a persistent container session.",
            inputSchema={
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "Container ID returned by start_session",
                    }
                },
                "required": ["container_id"],
            },
        ),
        types.Tool(
            name="open_debug_port",
            description=(
                "Start a container with a GDB stub port exposed to the host. "
                "Returns a container ID. The container stays running until stop_session is called. "
                "Connect your emulator's GDB stub to the exposed port, then attach from inside "
                "the container using debug-connect.sh."
            ),
            inputSchema={
                "type": "object",
                "properties": {
                    "platform": {
                        "type": "string",
                        "description": "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)",
                    },
                    "port": {
                        "type": "integer",
                        "description": "Port to expose (default: 2345)",
                        "default": 2345,
                    },
                },
                "required": ["platform"],
            },
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict) -> list[types.TextContent]:
    try:
        result = await _dispatch(name, arguments)
    except Exception as e:
        result = f"Error: {e}"
    return [types.TextContent(type="text", text=str(result))]


async def _dispatch(name: str, args: dict) -> str:
    if name == "list_platforms":
        lines = []
        for p, info in PLATFORMS.items():
            built = "✓" if _image_exists(info["image"]) else "✗"
            lines.append(f"{built} {p:6s}  ({info['image']})")
        return "\n".join(lines)

    if name == "get_platform_info":
        platform = args["platform"]
        if platform not in PLATFORMS:
            return f"Unknown platform: {platform}. Choose from: {', '.join(PLATFORMS)}"
        p_dir = _platform_dir(platform)
        dockerfile = (p_dir / "Dockerfile").read_text()
        launch = (p_dir / f"{PLATFORMS[platform]['image']}").read_text()
        return f"=== Dockerfile ===\n{dockerfile}\n=== Launch script ===\n{launch}"

    if name == "build_platform":
        platform = args["platform"]
        if platform not in PLATFORMS:
            return f"Unknown platform: {platform}"
        if not _image_exists("romhack-base"):
            result = subprocess.run(
                ["podman", "build", "-t", "romhack-base", str(REPO_ROOT / "shared")],
                capture_output=True, text=True, timeout=300,
            )
            if result.returncode != 0:
                return f"Failed to build romhack-base:\n{result.stderr}"
        p_dir = _platform_dir(platform)
        image = PLATFORMS[platform]["image"]
        result = subprocess.run(
            ["podman", "build", "-t", image, str(p_dir)],
            capture_output=True, text=True, timeout=3600,
        )
        if result.returncode != 0:
            return f"Build failed:\n{result.stderr}"
        return f"Built {image} successfully."

    if name == "run_command":
        platform = args["platform"]
        if platform not in PLATFORMS:
            return f"Unknown platform: {platform}"
        timeout = args.get("timeout", 60)
        cmd = _base_run_args(platform) + ["sh", "-c", args["command"]]
        result = subprocess.run(
            cmd, capture_output=True, text=True, timeout=timeout,
        )
        output = result.stdout
        if result.stderr:
            output += f"\n[stderr]\n{result.stderr}"
        if result.returncode != 0:
            output += f"\n[exit code: {result.returncode}]"
        return output or "(no output)"

    if name == "start_session":
        platform = args["platform"]
        if platform not in PLATFORMS:
            return f"Unknown platform: {platform}"
        p = PLATFORMS[platform]
        platform_dir = _platform_dir(platform)
        vendor_dir = platform_dir / "vendor"
        result = subprocess.run(
            [
                "podman", "run", "-d", "--rm",
                "--userns=keep-id:uid=1001,gid=1001",
                "-v", f"{vendor_dir}:{p['workdir']}/vendor",
                p["image"], "sleep", "infinity",
            ],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            return f"Failed to start session:\n{result.stderr}"
        container_id = result.stdout.strip()
        return f"Started session: {container_id}"

    if name == "run_in_session":
        container_id = args["container_id"]
        timeout = args.get("timeout", 60)
        result = subprocess.run(
            ["podman", "exec", container_id, "sh", "-c", args["command"]],
            capture_output=True, text=True, timeout=timeout,
        )
        output = result.stdout
        if result.stderr:
            output += f"\n[stderr]\n{result.stderr}"
        if result.returncode != 0:
            output += f"\n[exit code: {result.returncode}]"
        return output or "(no output)"

    if name == "stop_session":
        container_id = args["container_id"]
        result = subprocess.run(
            ["podman", "rm", "-f", container_id],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            return f"Failed to stop session:\n{result.stderr}"
        return f"Stopped session: {container_id}"

    if name == "open_debug_port":
        platform = args["platform"]
        if platform not in PLATFORMS:
            return f"Unknown platform: {platform}"
        port = args.get("port", 2345)
        p = PLATFORMS[platform]
        platform_dir = _platform_dir(platform)
        vendor_dir = platform_dir / "vendor"
        result = subprocess.run(
            [
                "podman", "run", "-d", "--rm",
                "--userns=keep-id:uid=1001,gid=1001",
                "-v", f"{vendor_dir}:{p['workdir']}/vendor",
                "-p", f"{port}:{port}",
                p["image"], "sleep", "infinity",
            ],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            return f"Failed to start debug session:\n{result.stderr}"
        container_id = result.stdout.strip()
        return (
            f"Debug session started: {container_id}\n"
            f"Port {port} is exposed on the host.\n"
            f"Start your emulator's GDB stub on port {port}, then from inside the container:\n"
            f"  ./debug-connect.sh host.gateway.internal {port}"
        )

    return f"Unknown tool: {name}"


async def main():
    async with mcp.server.stdio.stdio_server() as (read, write):
        await server.run(read, write, server.create_initialization_options())


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
