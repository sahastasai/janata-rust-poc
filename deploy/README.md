# Cloudflare Worker deployment boundary

This directory contains only the isolated `cmrust.sahasta.com` proof-of-concept
configuration. It has no Janata production bindings, resource IDs, or secrets.
Running a local build or Wrangler dry run does not deploy or change DNS.

## Static Dioxus artifact contract

`web/` is the upload staging directory expected by `wrangler.jsonc`. During
integration, replace the worker-spike smoke shell with the contents of the
Dioxus web release output while preserving `_headers`. Wrangler serves those
files directly from Cloudflare's static-asset layer; only `/api/*` invokes the
Rust WebAssembly Worker.

The checked-in smoke shell exists so configuration validation, local routing,
and dry runs are deterministic before the full Dioxus bundle is integrated.

## Local verification

From the repository root:

```sh
bun install
bun run worker:dry-run
bun run worker:startup
bun run worker:smoke
bun run worker:dev
```

Then request `http://127.0.0.1:8787/` and
`http://127.0.0.1:8787/api/health`. No credential is required for local mode.

Deployment is intentionally not scripted. An authorized release owner must
review the dry-run output and invoke Wrangler explicitly when the POC is ready.

## Panic-recovery compatibility note

The workspace release profile is `panic = "abort"`. `build-worker.sh` also uses
worker-build 0.8.5's `--no-panic-recovery` compatibility path because its new
catch-wrapper generator requires an externref table that Rust 1.96 does not
emit for this crate. Request handling avoids panics and returns structured 500s
for recoverable errors. Re-test the default worker-build path when either tool
is upgraded.
