import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dirname, "../../..");
const portText = process.env.JANATA_E2E_PORT ?? "8787";

if (!/^\d{4,5}$/.test(portText)) {
  throw new Error("JANATA_E2E_PORT must be a decimal loopback port from 1024 to 65535");
}

const port = Number(portText);
if (!Number.isSafeInteger(port) || port < 1_024 || port > 65_535) {
  throw new Error("JANATA_E2E_PORT must be between 1024 and 65535");
}

let build: ReturnType<typeof Bun.spawn> | undefined;
let wrangler: ReturnType<typeof Bun.spawn> | undefined;
let shuttingDown = false;

async function shutdown(signal: NodeJS.Signals): Promise<void> {
  if (shuttingDown) return;
  shuttingDown = true;
  build?.kill(signal);
  wrangler?.kill(signal);
  const forceKill = setTimeout(() => wrangler?.kill("SIGKILL"), 5_000);
  await Promise.allSettled([build?.exited, wrangler?.exited].filter(Boolean));
  clearTimeout(forceKill);
  process.exit(signal === "SIGINT" ? 130 : 143);
}

process.once("SIGINT", () => void shutdown("SIGINT"));
process.once("SIGTERM", () => void shutdown("SIGTERM"));

build = Bun.spawn(["./deploy/build-site.sh"], {
  cwd: repoRoot,
  env: process.env,
  stdin: "ignore",
  stdout: "inherit",
  stderr: "inherit",
});

const buildExitCode = await build.exited;
build = undefined;
if (buildExitCode !== 0) {
  throw new Error(`Production Dioxus site build exited with code ${buildExitCode}`);
}

// Wrangler serves the exact static-asset + workers-rs topology used by the
// deployment. This is important once Dioxus pages fetch same-origin /api data:
// a static-file substitute would turn real integration failures into mocks.
wrangler = Bun.spawn({
  cmd: [
    resolve(repoRoot, "node_modules/.bin/wrangler"),
    "dev",
    "--local",
    "--ip",
    "127.0.0.1",
    "--inspector-ip",
    "127.0.0.1",
    "--port",
    String(port),
    "--config",
    "deploy/wrangler.jsonc",
  ],
  cwd: repoRoot,
  env: process.env,
  stdin: "ignore",
  stdout: "inherit",
  stderr: "inherit",
});

const wranglerExitCode = await wrangler.exited;
wrangler = undefined;
if (!shuttingDown && wranglerExitCode !== 0) {
  throw new Error(`Loopback Wrangler exited with code ${wranglerExitCode}`);
}
