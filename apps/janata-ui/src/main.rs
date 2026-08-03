//! Dioxus entry point for the Janata proof of concept.

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

/// Renders the compile-gate shell used before feature work begins.
#[component]
fn App() -> Element {
    rsx! {
        main {
            h1 { "Janata — Rust proof of concept" }
            p { "Architecture feasibility gate in progress." }
        }
    }
}
