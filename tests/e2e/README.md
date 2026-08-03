# Dioxus web acceptance and parity harness

This harness has two deliberately different jobs:

- `e2e:current` verifies behavior the seven-screen Dioxus POC actually has.
- `e2e:parity` preserves the React reference acceptance contract as executable
  debt. It must not turn static preview content into a false parity claim.

Both commands build the production Dioxus, mdBook, and rustdoc output before a
test begins, then launch the real workers-rs bundle and assets through local
Wrangler. Wrangler and its inspector bind only to `127.0.0.1`; neither the host
nor base URL can be pointed at a deployment. Playwright owns server startup and
teardown. Set `JANATA_E2E_PORT` only when the default `8787` port is busy.

## Run it

Install the pinned dependencies and Chromium once, then run either contract:

```sh
bun install --frozen-lockfile
bunx playwright install chromium
bun run e2e:typecheck
bun run e2e:current
bun run e2e:parity
```

The projects are `desktop-web` at 1280×900 and `mobile-web` at 393×851. Every
test records console errors, uncaught page errors, and failed requests in a
`runtime-diagnostics.json` attachment; any entry fails the test.

Current-state coverage includes all seven canonical direct routes, the
`/discover` → `/explore` compatibility redirect, exactly one visible `h1`,
`aria-current`, document overflow, skip-link focus transfer, mobile bottom
navigation targets, and a complete 44×44 px mobile target audit. The last audit
is an expected failure while compact filters, feed actions, and inline links
remain undersized. If the CSS is fixed, Playwright reports an unexpected pass
until the `test.fail` annotation is removed.

## Frozen parity source

The 8 guest and 11 member cases are line-for-line expectation ports from:

- `Project-Janatha/tests/v2/guest.spec.ts`
- `Project-Janatha/tests/v2/member.spec.ts`
- reference commit `6e5bbacd2277b564a901e1eb4f48b566a1283802`

They run for both viewports: 38 expected failures today. `/explore` remains the
canonical path rather than being quietly translated in the contract. Every case uses
`test.fail(reason)` rather than a weakened assertion. A completed behavior is
therefore an unexpected pass and forces the debt annotation to be reviewed.

Member and guest cases first require this future product-owned adapter:

```ts
window.__JANATA_E2E__ = {
  async reset() {},
  async loginAs(role: "member") {},
  async state() {
    return {
      version: 1,
      authState: "guest" as "guest" | "member",
      dataSource: "live" as const,
      // Member state also returns referenceUser: "member".
    };
  },
};
```

The adapter must drive real application identity and live adapters. Do not
implement it as hand-authored local storage, and do not return `dataSource:
"live"` while rendering the compile-time Asha/San Jose sample cards. When a
case becomes real, run it on both projects, remove its `test.fail` annotation,
and keep its reference assertions intact.

HTML reports are written to `playwright-report/e2e`; traces, screenshots, and
attachments are written to `test-results/e2e`. Both directories are ignored.
