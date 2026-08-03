import type { Page } from "@playwright/test";
import { expect } from "./fixtures.ts";

type AuthState = "guest" | "member";

interface AdapterState {
  authState: AuthState;
  dataSource: "live";
  referenceUser?: "member";
  version: 1;
}

declare global {
  interface Window {
    __JANATA_E2E__?: {
      loginAs(role: "member"): Promise<void>;
      reset(): Promise<void>;
      state(): Promise<AdapterState>;
    };
  }
}

async function invokeAdapter(
  page: Page,
  operation: "guest" | "member",
): Promise<{ available: boolean; state?: AdapterState }> {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  return page.evaluate(async (requestedOperation) => {
    const adapter = window.__JANATA_E2E__;
    if (!adapter) return { available: false };
    await adapter.reset();
    if (requestedOperation === "member") await adapter.loginAs("member");
    return { available: true, state: await adapter.state() };
  }, operation);
}

/**
 * Establishes a real anonymous session through a future product-owned adapter.
 * Static compile-time cards cannot satisfy this contract.
 */
export async function requireGuestSession(page: Page): Promise<void> {
  const result = await invokeAdapter(page, "guest");
  expect(
    result.available,
    "guest parity requires window.__JANATA_E2E__; no production auth adapter is wired",
  ).toBe(true);
  expect(result.state).toEqual({ authState: "guest", dataSource: "live", version: 1 });
}

/**
 * Logs in as the reference member through a future product-owned adapter and
 * requires live data. Preview cards and hand-authored localStorage are not an
 * acceptable substitute for member parity.
 */
export async function requireReferenceMember(page: Page): Promise<void> {
  const result = await invokeAdapter(page, "member");
  expect(
    result.available,
    "member parity requires window.__JANATA_E2E__; no production auth adapter is wired",
  ).toBe(true);
  expect(result.state).toEqual({
    authState: "member",
    dataSource: "live",
    referenceUser: "member",
    version: 1,
  });
}

export {};
