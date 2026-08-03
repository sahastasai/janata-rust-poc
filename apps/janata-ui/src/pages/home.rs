use crate::Route;
use dioxus::prelude::*;

static SWAMI_CHINMAYANANDA: Asset = asset!("/assets/images/swami-chinmayananda.jpg");

/// Introduces the migration thesis and sends reviewers into the product shell.
#[component]
pub(crate) fn Home() -> Element {
    rsx! {
        title { "Janata — Rust proof of concept" }
        section { class: "hero",
            div { class: "hero-copy",
                p { class: "kicker", "CHINMAYA JANATA · RUST PROOF OF CONCEPT" }
                h1 {
                    "A community is not a feed."
                    span { " It is a practice." }
                }
                p { class: "hero-summary",
                    "One shared Dioxus interface for finding gatherings, following center life, and staying connected — designed to run on web and mobile from a Rust codebase."
                }
                div { class: "hero-actions",
                    Link { class: "button button-primary", to: Route::Discover, "Enter the POC" }
                    Link { class: "button button-quiet", to: Route::Docs, "Read the contributor note" }
                }
                ul { class: "proof-list", "aria-label": "Proof of concept boundaries",
                    li { "Type-safe routes" }
                    li { "Read-only Worker APIs proven" }
                    li { "Private reducers proven locally" }
                }
            }
            figure { class: "portrait-card",
                div { class: "portrait-thread", "aria-hidden": "true" }
                img {
                    src: SWAMI_CHINMAYANANDA,
                    alt: "Swami Chinmayananda seated outdoors in saffron robes",
                    width: "1057",
                    height: "1280",
                }
                figcaption {
                    strong { "Purpose before platform" }
                    span { "Swami Chinmayananda remains in view as the implementation changes." }
                }
            }
        }

        section { class: "home-evidence", "aria-labelledby": "gate-title",
            div { class: "evidence-intro",
                p { class: "kicker", "THE INTEGRATED POC" }
                h2 { id: "gate-title", "The foundation works. The boundaries stay visible." }
                p { "The shell now sits beside typed read-only Worker routes and locally proven private SpacetimeDB reducers. Sample community cards remain UI-only and are never presented as live accounts or messages." }
                div { class: "evidence-actions",
                    Link { to: Route::Benchmarks, "Review measured evidence →" }
                    Link { to: Route::Docs, "Open contributor docs →" }
                }
            }
            div { class: "gate-grid",
                article { class: "gate-card",
                    span { class: "gate-index", "IN" }
                    h3 { "Proven together" }
                    ul {
                        li { "Responsive Dioxus web and mobile feature builds" }
                        li { "Typed read-only Worker routes and contracts" }
                        li { "Private SpacetimeDB reducers validated locally" }
                    }
                }
                article { class: "gate-card gate-card-muted",
                    span { class: "gate-index", "GATE" }
                    h3 { "Required before beta" }
                    ul {
                        li { "Authenticated production identity and authorization" }
                        li { "Live UI adapters for Worker reads and subscriptions" }
                        li { "Native device distribution and end-to-end parity" }
                    }
                }
            }
        }
    }
}
