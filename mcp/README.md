# 123troh4x MCP Server

Exposes the platform containers as structured execution environments for AI
agents. The agent stays outside the container; this server is the bridge.

## Install

```bash
cd mcp
pip install -r requirements.txt
```

Or with uv:

```bash
uv pip install -r mcp/requirements.txt
```

## Tools

| Tool | Description |
|------|-------------|
| `list_platforms` | List platforms and whether their image is built |
| `get_platform_info` | Dockerfile + launch script for a platform |
| `build_platform` | Build a platform's container image |
| `run_command` | Run a one-shot command in a platform container |
| `start_session` | Start a persistent container, returns container ID |
| `run_in_session` | Run a command in a persistent session |
| `stop_session` | Stop and remove a persistent session |
| `open_debug_port` | Start a container with a GDB stub port exposed to the host |

## Usage with Claude Code

Add to your `CLAUDE.local.md` or configure in your MCP client:

```json
{
  "mcpServers": {
    "123troh4x": {
      "command": "python",
      "args": ["/path/to/romhack-playground/mcp/server.py"]
    }
  }
}
```

## Example

```
> list_platforms
✓ nes    (neshax)
✓ snes   (sneshax)
✓ gbc    (gbchax)
✓ gba    (gbahax)
✓ gen    (genhax)
✓ ds     (dshax)
✓ n64    (n64hax)
✓ ps1    (ps1hax)

> run_command platform=ds command="arm-none-eabi-gcc --version"
arm-none-eabi-gcc (15:13.2.rel1-2) 13.2.1 20231009

> start_session platform=ds
Started session: a3f2b1c4d5e6...

> run_in_session container_id=a3f2b1c4d5e6 command="cd vendor/pokeplatinum && make"
...build output...

> stop_session container_id=a3f2b1c4d5e6
Stopped session: a3f2b1c4d5e6
```

## Notes

- Runtime is `podman`. Docker is not supported.
- `vendor/` is mounted read-write so build artifacts persist between sessions.
- `open_debug_port` is for attaching a debugger from an emulator on the host.
  Start the emulator's GDB stub first, then connect from inside the container
  using `./debug-connect.sh`.
