import { expect, test as base } from "@playwright/test";

interface RuntimeDiagnostics {
  consoleErrors: string[];
  pageErrors: string[];
  requestFailures: string[];
}

/**
 * Extends every test with automatic browser-runtime diagnostics.
 *
 * The JSON attachment is retained even when no issue occurred. Any console
 * error, uncaught page error, or failed request also fails the test instead of
 * being buried in terminal output.
 */
export const test = base.extend<{ runtimeDiagnostics: void }>({
  runtimeDiagnostics: [
    async ({ page }, use, testInfo) => {
      const diagnostics: RuntimeDiagnostics = {
        consoleErrors: [],
        pageErrors: [],
        requestFailures: [],
      };

      page.on("console", (message) => {
        if (message.type() === "error") diagnostics.consoleErrors.push(message.text());
      });
      page.on("pageerror", (error) => diagnostics.pageErrors.push(error.stack ?? error.message));
      page.on("requestfailed", (request) => {
        const failure = request.failure()?.errorText ?? "unknown failure";
        diagnostics.requestFailures.push(`${request.method()} ${request.url()} — ${failure}`);
      });

      await use();

      await testInfo.attach("runtime-diagnostics.json", {
        body: Buffer.from(JSON.stringify(diagnostics, null, 2)),
        contentType: "application/json",
      });
      expect(diagnostics.consoleErrors, "browser console errors").toEqual([]);
      expect(diagnostics.pageErrors, "uncaught browser page errors").toEqual([]);
      expect(diagnostics.requestFailures, "failed browser requests").toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };
