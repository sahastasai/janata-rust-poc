# How the system fits together

The shortest useful mental model is “one domain, three runtimes.”

```text
Web browser or mobile WebView
        │
        │ shared Serde contracts
        ▼
Dioxus UI ───────────────► Rust Cloudflare Worker ─────► D1 / R2
   │                              │
   │ generated Rust bindings      │ short-lived identity bridge (gate)
   ▼                              ▼
SpacetimeDB client ─────────► SpacetimeDB messaging module
```

## Request lifecycle

1. A Dioxus route turns user input into a typed request from
   `janata-api-contract`.
2. The Worker assigns a request ID, applies security headers and exact-origin
   CORS, authenticates the session, and checks the named domain capability.
3. The route parses and validates input before using a Cloudflare binding.
4. The Worker serializes a typed response or a stable `ApiError`.
5. Dioxus renders success, loading, empty, offline, and failure states without
   learning database details.

Messaging is different: the client subscribes to caller-filtered SpacetimeDB
views and invokes reducers. Private source tables are never subscribed to
directly. Reducers derive identity from the authenticated context and repeat
authorization checks for every mutation.

## Why client-rendered Dioxus first

Dioxus Fullstack and `workers-rs` each support relevant WASM/Axum pieces, but
there is no official end-to-end Dioxus Fullstack-on-Workers adapter in the
architecture baseline. A client-rendered WASM application plus an explicit API
keeps deployment and debugging observable while still sharing Rust types.
