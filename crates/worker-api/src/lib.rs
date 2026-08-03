//! Cloudflare Worker API for the isolated Janata Rust proof of concept.

/// Returns the compile-gate service label on non-Wasm test hosts.
pub fn service_name() -> &'static str {
    "janata-rust-poc"
}
