# ADR 0001: Compile-first Rust POC architecture

Status: accepted for feasibility testing on 2026-08-03.

## Decision

Use a Dioxus 0.7.10 client-rendered WASM application for the web, the shared
Dioxus WebView renderer for mobile, a Rust `workers-rs` Cloudflare API, and a
separate SpacetimeDB service for real-time messaging.

SpacetimeDB will not run inside a Cloudflare Worker or an ephemeral Cloudflare
Container. The deployed POC targets Maincloud; local development uses a local
SpacetimeDB process.

## Consequences

- Web and mobile share Rust components and domain logic.
- Native maps, notifications, secure storage, media, location, and deep links
  require explicit platform bridges and dedicated device tests.
- Browser and Worker runtimes still include generated JavaScript/WASM glue.
- Performance claims require controlled measurements; Rust is not presumed to
  outperform React on every metric.

## Reference baseline

The frozen behavior reference is Project-Janatha commit
`6e5bbacd2277b564a901e1eb4f48b566a1283802`. No files or infrastructure in that
repository are changed by this POC.
