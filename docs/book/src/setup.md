# Five-minute setup

## Prerequisites

- Rust 1.96 through `rustup`
- Bun 1.3 or newer for Wrangler and browser-test tooling
- the Dioxus CLI matching the pinned Dioxus 0.7 release
- the `wasm32-unknown-unknown` Rust target

From the repository root:

```bash
rustup show
rustup target add wasm32-unknown-unknown
cargo check --workspace
cargo test --workspace
dx build --platform web --release
```

Install JavaScript tooling with Bun only:

```bash
bun install --frozen-lockfile
bunx wrangler --version
```

The repository intentionally avoids npm, npx, pnpm, and Yarn. They produce
different lock and resolution behavior from the workspace standard.

## Start the web UI

```bash
dx serve --platform web
```

The exact local URL appears in the command output. API work normally runs a
second terminal with the POC Wrangler configuration:

```bash
bunx wrangler dev --config deploy/wrangler.jsonc
```

Local secrets belong in an ignored `.dev.vars` file. Never copy the current
Janata production resources or values into this POC.

## Before opening a review

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

Run target-specific checks separately. Web and mobile renderer features are
intentionally mutually exclusive.
