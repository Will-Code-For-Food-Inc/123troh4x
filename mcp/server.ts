/**
 * 123troh4x MCP server
 *
 * Exposes the platform containers as structured execution environments for AI
 * agents. The agent stays outside the container; this server is the bridge.
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  type Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { spawnSync } from "child_process";
import { readFileSync } from "fs";
import { join } from "path";

// ---- Types ----

const PLATFORMS = {
  nes:  { image: "neshax",  workdir: "/neshax"  },
  snes: { image: "sneshax", workdir: "/sneshax" },
  gbc:  { image: "gbchax",  workdir: "/gbchax"  },
  gba:  { image: "gbahax",  workdir: "/gbahax"  },
  gen:  { image: "genhax",  workdir: "/genhax"  },
  ds:   { image: "dshax",   workdir: "/dshax"   },
  n64:  { image: "n64hax",  workdir: "/n64hax"  },
  ps1:  { image: "ps1hax",  workdir: "/ps1hax"  },
} as const satisfies Record<string, { image: string; workdir: string }>;

type Platform = keyof typeof PLATFORMS;
type PlatformInfo = (typeof PLATFORMS)[Platform];

type RunResult = { stdout: string; stderr: string; code: number };

type ToolArgs =
  | { tool: "list_platforms" }
  | { tool: "get_platform_info"; platform: Platform }
  | { tool: "build_platform"; platform: Platform }
  | { tool: "run_command"; platform: Platform; command: string; timeout?: number }
  | { tool: "start_session"; platform: Platform }
  | { tool: "run_in_session"; container_id: string; command: string; timeout?: number }
  | { tool: "stop_session"; container_id: string }
  | { tool: "open_debug_port"; platform: Platform; port?: number };

// ---- Constants ----

const REPO_ROOT = join(import.meta.dir, "..");
const DEFAULT_TIMEOUT_MS = 60_000;
const BUILD_TIMEOUT_MS = 3_600_000;

// ---- Pure helpers ----

const isPlatform = (s: unknown): s is Platform => typeof s === "string" && s in PLATFORMS;

const platformDir = (platform: Platform): string =>
  join(REPO_ROOT, "platforms", platform);

const imageExists = (image: string): boolean =>
  spawnSync("podman", ["image", "exists", image]).status === 0;

const podman = (args: string[], timeoutMs = DEFAULT_TIMEOUT_MS): RunResult => {
  const r = spawnSync("podman", args, { encoding: "utf8", timeout: timeoutMs });
  return { stdout: r.stdout ?? "", stderr: r.stderr ?? "", code: r.status ?? 1 };
};

const formatOutput = ({ stdout, stderr, code }: RunResult): string => {
  const parts = [stdout, stderr && `[stderr]\n${stderr}`, code !== 0 && `[exit code: ${code}]`];
  return parts.filter(Boolean).join("\n").trim() || "(no output)";
};

const vendorMount = (platform: Platform): string => {
  const { workdir } = PLATFORMS[platform];
  return `${join(platformDir(platform), "vendor")}:${workdir}/vendor`;
};

const baseRunArgs = (platform: Platform): string[] => [
  "run", "--rm",
  "--userns=keep-id:uid=1001,gid=1001",
  "-v", vendorMount(platform),
  PLATFORMS[platform].image,
];

const parseArgs = (name: string, raw: Record<string, unknown>): ToolArgs | { error: string } => {
  switch (name) {
    case "list_platforms":
      return { tool: "list_platforms" };

    case "get_platform_info":
    case "build_platform":
    case "start_session":
    case "open_debug_port": {
      if (!isPlatform(raw.platform)) return { error: `Unknown platform: ${raw.platform}. Valid: ${Object.keys(PLATFORMS).join(", ")}` };
      if (name === "open_debug_port") return { tool: "open_debug_port", platform: raw.platform, port: raw.port as number | undefined };
      return { tool: name as "get_platform_info" | "build_platform" | "start_session", platform: raw.platform };
    }

    case "run_command": {
      if (!isPlatform(raw.platform)) return { error: `Unknown platform: ${raw.platform}` };
      if (typeof raw.command !== "string") return { error: "command must be a string" };
      return { tool: "run_command", platform: raw.platform, command: raw.command, timeout: raw.timeout as number | undefined };
    }

    case "run_in_session": {
      if (typeof raw.container_id !== "string") return { error: "container_id required" };
      if (typeof raw.command !== "string") return { error: "command required" };
      return { tool: "run_in_session", container_id: raw.container_id, command: raw.command, timeout: raw.timeout as number | undefined };
    }

    case "stop_session": {
      if (typeof raw.container_id !== "string") return { error: "container_id required" };
      return { tool: "stop_session", container_id: raw.container_id };
    }

    default:
      return { error: `Unknown tool: ${name}` };
  }
};

// ---- Tool handlers ----
// Each returns a plain string result.

const handleListPlatforms = (): string =>
  Object.entries(PLATFORMS)
    .map(([p, { image }]) => `${imageExists(image) ? "✓" : "✗"} ${p.padEnd(6)}  (${image})`)
    .join("\n");

const handleGetPlatformInfo = (platform: Platform): string => {
  const dir = platformDir(platform);
  const dockerfile   = readFileSync(join(dir, "Dockerfile"), "utf8");
  const launchScript = readFileSync(join(dir, PLATFORMS[platform].image), "utf8");
  return `=== Dockerfile ===\n${dockerfile}\n=== Launch script ===\n${launchScript}`;
};

const handleBuildPlatform = (platform: Platform): string => {
  if (!imageExists("romhack-base")) {
    const r = podman(["build", "-t", "romhack-base", join(REPO_ROOT, "shared")], 300_000);
    if (r.code !== 0) return `Failed to build romhack-base:\n${r.stderr}`;
  }
  const r = podman(["build", "-t", PLATFORMS[platform].image, platformDir(platform)], BUILD_TIMEOUT_MS);
  return r.code !== 0
    ? `Build failed:\n${r.stderr}`
    : `Built ${PLATFORMS[platform].image} successfully.`;
};

const handleRunCommand = (platform: Platform, command: string, timeout?: number): string =>
  formatOutput(podman([...baseRunArgs(platform), "sh", "-c", command], (timeout ?? 60) * 1000));

const handleStartSession = (platform: Platform): string => {
  const r = podman([
    "run", "-d", "--rm",
    "--userns=keep-id:uid=1001,gid=1001",
    "-v", vendorMount(platform),
    PLATFORMS[platform].image, "sleep", "infinity",
  ]);
  return r.code !== 0
    ? `Failed to start session:\n${r.stderr}`
    : `Started session: ${r.stdout.trim()}`;
};

const handleRunInSession = (container_id: string, command: string, timeout?: number): string =>
  formatOutput(podman(["exec", container_id, "sh", "-c", command], (timeout ?? 60) * 1000));

const handleStopSession = (container_id: string): string => {
  const r = podman(["rm", "-f", container_id]);
  return r.code !== 0
    ? `Failed to stop session:\n${r.stderr}`
    : `Stopped session: ${container_id}`;
};

const handleOpenDebugPort = (platform: Platform, port = 2345): string => {
  const r = podman([
    "run", "-d", "--rm",
    "--userns=keep-id:uid=1001,gid=1001",
    "-v", vendorMount(platform),
    "-p", `${port}:${port}`,
    PLATFORMS[platform].image, "sleep", "infinity",
  ]);
  if (r.code !== 0) return `Failed to start debug session:\n${r.stderr}`;
  return [
    `Debug session started: ${r.stdout.trim()}`,
    `Port ${port} is exposed on the host.`,
    `Start your emulator's GDB stub on port ${port}, then from inside the container:`,
    `  ./debug-connect.sh host.gateway.internal ${port}`,
  ].join("\n");
};

// ---- Dispatch ----

const dispatch = (args: ToolArgs): string => {
  switch (args.tool) {
    case "list_platforms":    return handleListPlatforms();
    case "get_platform_info": return handleGetPlatformInfo(args.platform);
    case "build_platform":    return handleBuildPlatform(args.platform);
    case "run_command":       return handleRunCommand(args.platform, args.command, args.timeout);
    case "start_session":     return handleStartSession(args.platform);
    case "run_in_session":    return handleRunInSession(args.container_id, args.command, args.timeout);
    case "stop_session":      return handleStopSession(args.container_id);
    case "open_debug_port":   return handleOpenDebugPort(args.platform, args.port);
  }
};

// ---- Tool definitions ----

const TOOLS: Tool[] = [
  {
    name: "list_platforms",
    description: "List all available platforms and whether their container image is built.",
    inputSchema: { type: "object", properties: {}, required: [] },
  },
  {
    name: "get_platform_info",
    description: "Get Dockerfile and launch script contents for a platform.",
    inputSchema: {
      type: "object",
      properties: { platform: { type: "string", description: "Platform name (nes, snes, gbc, gba, gen, ds, n64, ps1)" } },
      required: ["platform"],
    },
  },
  {
    name: "build_platform",
    description: "Build the container image for a platform. Builds romhack-base first if needed.",
    inputSchema: {
      type: "object",
      properties: { platform: { type: "string" } },
      required: ["platform"],
    },
  },
  {
    name: "run_command",
    description: "Run a one-shot command inside a platform container and return output. vendor/ is mounted read-write.",
    inputSchema: {
      type: "object",
      properties: {
        platform: { type: "string" },
        command:  { type: "string", description: "Shell command to run inside the container" },
        timeout:  { type: "number", description: "Timeout in seconds (default: 60)" },
      },
      required: ["platform", "command"],
    },
  },
  {
    name: "start_session",
    description: "Start a persistent detached container session. Returns a container ID for use with run_in_session.",
    inputSchema: {
      type: "object",
      properties: { platform: { type: "string" } },
      required: ["platform"],
    },
  },
  {
    name: "run_in_session",
    description: "Run a command in an existing persistent container session.",
    inputSchema: {
      type: "object",
      properties: {
        container_id: { type: "string", description: "Container ID from start_session" },
        command:      { type: "string" },
        timeout:      { type: "number", description: "Timeout in seconds (default: 60)" },
      },
      required: ["container_id", "command"],
    },
  },
  {
    name: "stop_session",
    description: "Stop and remove a persistent container session.",
    inputSchema: {
      type: "object",
      properties: { container_id: { type: "string" } },
      required: ["container_id"],
    },
  },
  {
    name: "open_debug_port",
    description:
      "Start a container with a GDB stub port exposed to the host. " +
      "Connect your emulator's GDB stub to the exposed port, then attach from inside using debug-connect.sh.",
    inputSchema: {
      type: "object",
      properties: {
        platform: { type: "string" },
        port:     { type: "number", description: "Port to expose (default: 2345)" },
      },
      required: ["platform"],
    },
  },
];

// ---- Wire up ----

const server = new Server(
  { name: "123troh4x", version: "0.1.0" },
  { capabilities: { tools: {} } },
);

server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools: TOOLS }));

server.setRequestHandler(CallToolRequestSchema, async (req) => {
  const parsed = parseArgs(req.params.name, (req.params.arguments ?? {}) as Record<string, unknown>);
  if ("error" in parsed) {
    return { content: [{ type: "text", text: parsed.error }], isError: true };
  }
  try {
    return { content: [{ type: "text", text: dispatch(parsed) }] };
  } catch (e) {
    return { content: [{ type: "text", text: `Error: ${e}` }], isError: true };
  }
});

await server.connect(new StdioServerTransport());
