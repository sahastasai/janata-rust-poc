import type { Page } from "@playwright/test";
import { expect, test } from "../support/fixtures.ts";
import { gotoApp } from "../support/navigation.ts";

const routes = [
  { path: "/", label: "Home", heading: /A community is not a feed/i },
  { path: "/explore", label: "Discover", heading: /Find what brings you together/i },
  { path: "/feed", label: "Feed", heading: /What your circles are sharing/i },
  { path: "/connect", label: "Connect", heading: /Conversations with context/i },
  { path: "/profile", label: "Profile", heading: /Your place in the community/i },
  { path: "/benchmarks", label: "Benchmarks", heading: /The local result is mixed/i },
  { path: "/docs", label: "Contributor docs", heading: /New to Rust/i },
] as const;

async function overflowMetrics(page: Page): Promise<{ clientWidth: number; scrollWidth: number }> {
  return page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
}

test.describe("current Dioxus route contract", { tag: "@current" }, () => {
  for (const route of routes) {
    test(`${route.label} is a direct, single-H1, non-overflowing route`, async ({ page }) => {
      await gotoApp(page, route.path);

      expect(new URL(page.url()).pathname).toBe(route.path);
      const headings = page.getByRole("heading", { level: 1 });
      await expect(headings).toHaveCount(1);
      await expect(headings).toHaveText(route.heading);

      const activeRouteLink = page.locator(`a.nav-link[href="${route.path}"]`).first();
      await expect(activeRouteLink).toHaveAttribute("aria-current", "page");

      const dimensions = await overflowMetrics(page);
      expect(
        dimensions.scrollWidth,
        `${route.path} document should not exceed its viewport width`,
      ).toBeLessThanOrEqual(dimensions.clientWidth + 1);
    });
  }

  test("the skip link transfers keyboard focus to main content", async ({ page }) => {
    await gotoApp(page, "/explore");

    const skipLink = page.getByRole("link", { name: "Skip to content" });
    await page.keyboard.press("Tab");
    await expect(skipLink).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page.locator("#main-content")).toBeFocused();
  });

  test("the former /discover path redirects to canonical /explore", async ({ page }) => {
    await gotoApp(page, "/discover");
    await expect(page).toHaveURL(/\/explore$/);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(/Find what brings you together/i);
  });

  test("mobile bottom navigation exposes five 44px targets", async ({ page }, testInfo) => {
    test.skip(testInfo.project.name !== "mobile-web", "mobile-only target-size check");
    await gotoApp(page, "/");

    const targets = page.locator(".bottom-nav .nav-link");
    await expect(targets).toHaveCount(5);
    for (let index = 0; index < (await targets.count()); index += 1) {
      const target = targets.nth(index);
      const box = await target.boundingBox();
      expect(box, `bottom navigation target ${index + 1} should render`).not.toBeNull();
      expect(box?.width ?? 0, `bottom navigation target ${index + 1} width`).toBeGreaterThanOrEqual(44);
      expect(box?.height ?? 0, `bottom navigation target ${index + 1} height`).toBeGreaterThanOrEqual(44);
    }
  });

  test("every rendered mobile interactive target is at least 44px by 44px", async (
    { page },
    testInfo,
  ) => {
    test.skip(testInfo.project.name !== "mobile-web", "mobile-only target-size audit");
    test.fail(
      true,
      "Design debt: compact feed actions, filters, and inline documentation links still render below the 44px mobile target minimum",
    );

    const violations: string[] = [];
    for (const route of routes) {
      await gotoApp(page, route.path);
      const routeViolations = await page.locator(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [role="button"]:not([aria-disabled="true"])',
      ).evaluateAll((elements) =>
        elements.flatMap((element) => {
          const node = element as HTMLElement;
          const style = getComputedStyle(node);
          const box = node.getBoundingClientRect();
          if (
            style.display === "none" ||
            style.visibility === "hidden" ||
            box.width === 0 ||
            box.height === 0
          ) {
            return [];
          }
          if (box.width >= 44 && box.height >= 44) return [];
          const name =
            node.getAttribute("aria-label") ??
            node.textContent?.trim().replace(/\s+/g, " ") ??
            node.tagName.toLowerCase();
          return [`${name.slice(0, 70)} (${box.width.toFixed(1)}×${box.height.toFixed(1)})`];
        }),
      );
      violations.push(...routeViolations.map((violation) => `${route.path}: ${violation}`));
    }

    expect(violations, `undersized mobile targets:\n${violations.join("\n")}`).toEqual([]);
  });
});
