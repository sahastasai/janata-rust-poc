# Troubleshooting

## “cannot find macro `rsx`” or `dioxus::launch`

The Dioxus dependency is missing its `minimal`/macro/launch feature set. Check
the workspace dependency and the target feature before adding unrelated crates.

## UUID randomness error on `wasm32-unknown-unknown`

Shared domain IDs parse and serialize UUIDs but do not generate random IDs in
the browser. Keep generation on a server binding. Do not enable a random source
merely to make a target compile unless that runtime truly owns generation.

## A crate works on the host but not WebAssembly

Run the exact target check and inspect transitive features. Network, filesystem,
threading, and random-number assumptions are common causes. Isolate the crate
behind a platform adapter instead of enabling all features.

## Dioxus mobile fails on Linux

Check the Android SDK/NDK, Java, Rust Android targets, and emulator separately.
iOS cannot be built on Linux; use the macOS CI workflow.

## Wrangler cannot find a binding

Compare the binding name in `deploy/wrangler.jsonc` with Worker lookup code and
regenerate/inspect configuration. Never work around a missing binding with a
hard-coded Cloudflare REST token.

## The compiler says a SpacetimeDB table method is missing

Import the `spacetimedb::Table` trait. For views, remember that only indexed
lookups are allowed; add an evidence-based index or redesign the view instead
of scanning private tables.
