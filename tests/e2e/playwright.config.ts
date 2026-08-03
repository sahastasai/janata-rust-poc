import { defineConfig, devices } from "@playwright/test";
import { resolve } from "node:path";

// Playwright 1.58 loads TypeScript configuration through its CommonJS
// transformer, so use its provided directory rather than ESM import metadata.
const repoRoot = resolve(__dirname, "../..");
const portText = process.env.JANATA_E2E_PORT ?? "8787";

if (!/^\d{4,5}$/.test(portText)) {
  throw new Error("JANATA_E2E_PORT must be a decimal loopback port from 1024 to 65535");
}

const port = Number(portText);
if (!Number.isSafeInteger(port) || port < 1_024 || port > 65_535) {
  throw new Error("JANATA_E2E_PORT must be between 1024 and 65535");
}

// Intentionally not configurable: acceptance tests must never mutate or load-test
// a deployed environment by accident. The companion server binds to this exact
// IPv4 loopback address and builds the release output before accepting requests.
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: repoRoot + "/tests/e2e",
  outputDir: repoRoot + "/test-results/e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  workers: process.env.CI ? 2 : 2,
  reporter: [
    ["list"],
    ["html", { open: "never", outputFolder: repoRoot + "/playwright-report/e2e" }],
  ],
  timeout: 45_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL,
    actionTimeout: 10_000,
    navigationTimeout: 30_000,
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
    video: "off",
  },
  webServer: {
    command: "bun tests/e2e/support/production-server.ts",
    cwd: repoRoot,
    env: { ...process.env, JANATA_E2E_PORT: String(port) },
    reuseExistingServer: false,
    timeout: 360_000,
    url: `${baseURL}/api/health`,
    stdout: "pipe",
    stderr: "pipe",
  },
  projects: [
    {
      name: "desktop-web",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1_280, height: 900 },
      },
    },
    {
      name: "mobile-web",
      use: {
        ...devices["Pixel 5"],
        viewport: { width: 393, height: 851 },
      },
    },
  ],
});
