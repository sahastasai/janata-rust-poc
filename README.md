# Janata Dioxus proof of concept

An independent, non-production proof of concept for moving Project Janatha from
Expo/React Native and Hono to a Rust-first stack built with Dioxus,
Cloudflare Workers, and SpacetimeDB messaging.

> Om Sri Chinmaya Sadgurave Namaha. Om Sri Gurubyo Namaha.

## Safety boundary

- The reference application remains untouched at Project-Janatha commit
  `6e5bbacd2277b564a901e1eb4f48b566a1283802`.
- This repository owns only POC infrastructure and must never bind to existing
  Janata production databases, buckets, Workers, mobile apps, or deployment
  pipelines.
- Deployment is limited to the isolated `cmrust.sahasta.com` POC domain.
- Mobile outputs are development/beta artifacts, not store submissions.

## Architecture direction

- `apps/janata-ui`: shared Dioxus web and mobile user interface.
- `crates/domain`: framework-independent entities, validation, and permissions.
- `crates/api-contract`: Serde request and response contracts.
- `crates/worker-api`: Rust Cloudflare Worker API and static-asset entry point.
- `crates/spacetime-module`: real-time messaging tables, views, and reducers.
- `docs/book`: newcomer-focused contributor documentation.

The product code is Rust. Generated `wasm-bindgen` JavaScript, Wrangler/Bun
tooling, and Kotlin/Swift platform bridges are documented exceptions required by
browser and mobile operating-system runtimes.

## Status

The repository pins stable Dioxus 0.7.10 and is in its compile-first feasibility phase. A feature is not
considered migrated until its contract, role matrix, responsive UI, security
tests, and performance evidence are present.
