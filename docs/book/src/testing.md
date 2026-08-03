# Testing

## Fast checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Web and Worker targets

```bash
cargo check -p janata-ui --target wasm32-unknown-unknown \
  --no-default-features --features web
cargo check -p janata-worker-api --target wasm32-unknown-unknown
dx build --platform web --release
bunx wrangler deploy --dry-run --config deploy/wrangler.jsonc
```

## What different tests prove

- Domain tests prove validation and permissions without a framework.
- Contract tests prove JSON shape and status compatibility.
- Worker tests prove routing, headers, bindings, and server authorization.
- SpacetimeDB tests prove reducers and caller isolation.
- Browser tests prove route navigation, responsive interaction, and
  accessibility signals.
- Native tests prove permissions, bridges, lifecycle, safe areas, and
  platform packaging.
- Load tests measure capacity and tail latency; they do not prove correctness.

Keep fixtures deterministic. Never point tests or load tools at production
Janata resources.
