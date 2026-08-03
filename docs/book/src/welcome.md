# Welcome

This guide is for anyone who wants to help with the Janata Rust proof of
concept. You do **not** need to arrive as a Rust expert. The examples explain
the project-specific patterns, point to the file that owns each decision, and
show the check that proves a change works.

> Om Sri Chinmaya Sadgurave Namaha. Om Sri Gurubyo Namaha.

The product connects members of the Chinmaya community with centers, events,
boards, notifications, and one another. In the spirit of Swami Chinmayananda,
the code should turn understanding into service: clear interfaces, disciplined
tests, accessible experiences, and documentation that invites the next person
to contribute.

## What this repository is

This is an isolated proof of concept. It references Project-Janatha commit
`6e5bbac`, but it has separate Git history and separate infrastructure. It must
not modify the current production application, database, storage, mobile-store
listings, or deployment pipeline.

The application code is Rust:

- Dioxus renders the web and shared mobile interface.
- `workers-rs` runs the API on Cloudflare Workers.
- SpacetimeDB stores and synchronizes real-time messages.
- Serde contracts and the domain crate keep those boundaries consistent.

Browsers and operating systems still require generated JavaScript/WASM glue,
Wrangler tooling, and small Kotlin/Swift bridges. The project calls these out
instead of describing them as Rust.

## A completion claim needs evidence

A screen looking correct is useful, but it is not enough. A migrated feature
also needs its API contract, permissions, error and empty states, responsive
behavior, tests, and performance measurements. The parity manifests under
`docs/parity/` keep the reference surface visible while the POC grows.
