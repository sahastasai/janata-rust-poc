import { expect, test } from "../support/fixtures.ts";
import { visibleText } from "../support/navigation.ts";
import { requireGuestSession } from "../support/parity-adapter.ts";

// Exact guest acceptance contract ported from Project-Janatha
// tests/v2/guest.spec.ts at 6e5bbacd2277b564a901e1eb4f48b566a1283802.
// Routes are intentionally not translated (/explore is not /discover).
test.describe("reference guest acceptance contract", { tag: "@parity" }, () => {
  test("intro carousel renders the first slide", async ({ page }) => {
    test.fail(true, "Parity debt: the Dioxus app has no guest auth adapter or /intro carousel route");
    await requireGuestSession(page);
    await page.goto("/intro");
    await expect(visibleText(page, /Find your center/i)).toBeVisible();
    await expect(visibleText(page, /Grow together/i)).toBeVisible();
    await expect(visibleText(page, /^Next$/i)).toBeVisible();
  });

  test("auth screen shows the welcome heading + invite affordance", async ({ page }) => {
    test.fail(true, "Parity debt: invite-only guest authentication and /auth are not implemented");
    await requireGuestSession(page);
    await page.goto("/auth");
    await expect(page.getByRole("heading", { name: /Welcome/i })).toBeVisible();
    await expect(visibleText(page, /Have an invite/i)).toBeVisible();
  });

  test("auth screen has the email entry", async ({ page }) => {
    test.fail(true, "Parity debt: /auth has no production email-entry flow");
    await requireGuestSession(page);
    await page.goto("/auth");
    await expect(page.getByPlaceholder(/email/i)).toBeVisible();
  });

  test("guest Home shows the sign-in nudge", async ({ page }) => {
    test.fail(true, "Parity debt: Home has no real guest/member state and no sign-in nudge");
    await requireGuestSession(page);
    await page.goto("/");
    await expect(visibleText(page, /Log in to make Janata yours|Make Janata yours/i)).toBeVisible();
  });

  test("guest Home hides personalized event lists (#399)", async ({ page }) => {
    test.fail(
      true,
      "Parity debt: absence of static headings is not guest parity until a live guest adapter proves the session state",
    );
    await requireGuestSession(page);
    await page.goto("/");
    await expect(page.getByText(/UP NEXT FOR YOU/i).filter({ visible: true })).toHaveCount(0);
    await expect(page.getByText(/COMING UP/i).filter({ visible: true })).toHaveCount(0);
  });

  test("guest Feed shows the setup rail", async ({ page }) => {
    test.fail(true, "Parity debt: Feed has preview posts instead of the guest login/setup rail");
    await requireGuestSession(page);
    await page.goto("/feed");
    await expect(visibleText(page, /Log in to see your feed/i)).toBeVisible();
    await expect(visibleText(page, /Log in/i)).toBeVisible();
  });

  test("guest Feed keeps the preview quiet", async ({ page }) => {
    test.fail(
      true,
      "Parity debt: Feed is a static preview; a live guest adapter must prove guest-specific visibility",
    );
    await requireGuestSession(page);
    await page.goto("/feed");
    await expect(page.getByText(/Your community feed/i).filter({ visible: true })).toHaveCount(0);
  });

  test("guest can browse Explore (centers + events)", async ({ page }) => {
    test.fail(true, "Parity debt: the exact /explore guest route and live center/event reads are missing");
    await requireGuestSession(page);
    await page.goto("/explore");
    await expect(visibleText(page, /Chinmaya/i)).toBeVisible();
  });
});
