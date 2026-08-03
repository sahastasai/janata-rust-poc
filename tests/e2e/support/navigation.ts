import type { Locator, Page } from "@playwright/test";
import { expect } from "./fixtures.ts";

export async function gotoApp(page: Page, path: string): Promise<void> {
  const response = await page.goto(path, { waitUntil: "domcontentloaded" });
  expect(response, `direct navigation to ${path} should return a document`).not.toBeNull();
  expect(response?.ok(), `direct navigation to ${path} should be successful`).toBe(true);
  await expect(page.locator("#main-content")).toBeVisible();
  await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
}

export function visibleText(page: Page, pattern: RegExp): Locator {
  return page.getByText(pattern).filter({ visible: true }).first();
}
