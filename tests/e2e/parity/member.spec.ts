import { expect, test } from "../support/fixtures.ts";
import { visibleText } from "../support/navigation.ts";
import { requireReferenceMember } from "../support/parity-adapter.ts";

// Exact member acceptance contract ported from Project-Janatha
// tests/v2/member.spec.ts at 6e5bbacd2277b564a901e1eb4f48b566a1283802.
// The adapter requires live data so sample Asha/San Jose cards can never satisfy
// the seeded member@ / Chinmaya Vrindavan contract by coincidence.
const VRINDAVAN = "c0000001-0000-0000-0000-000000000081";

test.describe("reference member acceptance contract", { tag: "@parity" }, () => {
  test("Home greets the member by name", async ({ page }) => {
    test.fail(true, "Parity debt: production member identity and the personalized Home greeting are absent");
    await requireReferenceMember(page);
    await page.goto("/");
    await expect(visibleText(page, /Namaste/i)).toBeVisible();
  });

  test("Home shows role + center personalization", async ({ page }) => {
    test.fail(true, "Parity debt: Home is not connected to the reference member's live center or role");
    await requireReferenceMember(page);
    await page.goto("/");
    await expect(visibleText(page, /Chinmaya Vrindavan/i)).toBeVisible();
  });

  test("Explore picker defaults to the member center", async ({ page }) => {
    test.fail(true, "Parity debt: /explore and its authenticated center picker are not implemented");
    await requireReferenceMember(page);
    await page.goto("/explore");
    await expect(visibleText(page, /Chinmaya Vrindavan/i)).toBeVisible();
  });

  test("Explore lists upcoming events", async ({ page }) => {
    test.fail(true, "Parity debt: /explore has no live upcoming-event adapter");
    await requireReferenceMember(page);
    await page.goto("/explore");
    await expect(visibleText(page, /Bala Vihar|Gita Study|Mahasamadhi/i)).toBeVisible();
  });

  test("Feed shows the populated board posts", async ({ page }) => {
    test.fail(true, "Parity debt: static preview posts are not the reference member's live board feed");
    await requireReferenceMember(page);
    await page.goto("/feed");
    await expect(visibleText(page, /Vrindavan board|satsang|South Bay/i)).toBeVisible();
  });

  test("Center detail shows the center header", async ({ page }) => {
    test.fail(true, "Parity debt: the parameterized /center/:id route is missing");
    await requireReferenceMember(page);
    await page.goto(`/center/${VRINDAVAN}`);
    await expect(visibleText(page, /Chinmaya Vrindavan/i)).toBeVisible();
  });

  test("Center detail renders the board posts", async ({ page }) => {
    test.fail(true, "Parity debt: center detail has no live member-authorized board adapter");
    await requireReferenceMember(page);
    await page.goto(`/center/${VRINDAVAN}`);
    await expect(visibleText(page, /Vrindavan board|satsang|South Bay/i)).toBeVisible();
  });

  test("Settings has the Profile section as the top item", async ({ page }) => {
    test.fail(true, "Parity debt: authenticated /settings and member profile settings are missing");
    await requireReferenceMember(page);
    await page.goto("/settings");
    await expect(visibleText(page, /^Profile$/i)).toBeVisible();
    await expect(visibleText(page, /Member Demo/i)).toBeVisible();
  });

  test("Settings links to notification preferences (consolidated page)", async ({ page }) => {
    test.fail(true, "Parity debt: /settings does not expose notification preferences");
    await requireReferenceMember(page);
    await page.goto("/settings");
    await expect(visibleText(page, /Notification preferences/i)).toBeVisible();
  });

  test("Notification preferences shows the channels", async ({ page }) => {
    test.fail(true, "Parity debt: /settings/notifications and persisted channel preferences are missing");
    await requireReferenceMember(page);
    await page.goto("/settings/notifications");
    await expect(visibleText(page, /Notification Channels/i)).toBeVisible();
  });

  test("Notification preferences shows the per-type toggles", async ({ page }) => {
    test.fail(true, "Parity debt: per-type member notification preferences are not implemented");
    await requireReferenceMember(page);
    await page.goto("/settings/notifications");
    await expect(visibleText(page, /Event Reminders/i)).toBeVisible();
  });
});
