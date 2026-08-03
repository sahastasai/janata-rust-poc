# Cloudflare Worker deployment boundary

This directory contains only the isolated `cmrust.sahasta.com` proof-of-concept
configuration. It has no Janata production bindings, resource IDs, or secrets.
Running a local build or Wrangler dry run does not deploy or change DNS.

## Static Dioxus artifact contract

`../dist/site/` is the generated upload staging directory expected by
`wrangler.jsonc`. `bun run poc:build` assembles the Dioxus web release, mdBook
guide, rustdoc API reference, and static security headers there. It excludes
Dioxus `.br` sidecars because Cloudflare negotiates edge compression itself.
Wrangler serves these files directly from Cloudflare's static-asset layer;
only `/api/*` invokes the Rust WebAssembly Worker.

## Local verification

From the repository root:

```sh
bun install
bun run poc:build
bun run worker:dry-run
bun run worker:startup
bun run worker:smoke
bun run worker:dev
```

Then request `http://127.0.0.1:8787/` and
`http://127.0.0.1:8787/api/health`. No credential is required for local mode.

`bun run worker:deploy:preview` publishes only the isolated `workers.dev`
preview. After that preview passes smoke testing, the separately reviewed
`wrangler.production.jsonc` configuration can attach only
`cmrust.sahasta.com`. Neither configuration contains production Janata
bindings or secrets.

## Panic-recovery compatibility note

The workspace release profile is `panic = "abort"`. `build-worker.sh` also uses
worker-build 0.8.5's `--no-panic-recovery` compatibility path because its new
catch-wrapper generator requires an externref table that Rust 1.96 does not
emit for this crate. Request handling avoids panics and returns structured 500s
for recoverable errors. Re-test the default worker-build path when either tool
is upgraded.
