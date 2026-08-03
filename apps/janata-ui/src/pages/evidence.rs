use crate::{
    Route,
    components::{CheckRow, PageHeader},
};
use dioxus::prelude::*;

/// Presents measured local migration evidence, including unfavorable results.
#[component]
pub(crate) fn Benchmarks() -> Element {
    rsx! {
        title { "Benchmarks — Janata Rust POC" }
        div { class: "page evidence-page",
            PageHeader {
                kicker: "MIGRATION EVIDENCE",
                title: "The local result is mixed — and useful.",
                summary: "Dioxus produced a much smaller logical web release and lower 30-run landing LCP medians. The workers-rs health path was materially slower than the current Hono baseline.",
                status: "Measured locally · August 2026",
            }
            section { class: "verdict-grid", "aria-label": "Benchmark result summary",
                article { class: "verdict verdict-dioxus",
                    p { class: "utility-label", "DIOXUS LEADS LOCALLY" }
                    h2 { "Web release size and landing LCP" }
                    p { "The Rust UI won both measured browser categories on this machine." }
                }
                article { class: "verdict verdict-react",
                    p { class: "utility-label", "REACT / HONO LEADS LOCALLY" }
                    h2 { "Worker health throughput and p95" }
                    p { "The existing Hono health endpoint was roughly an order of magnitude faster in this local microbenchmark." }
                }
            }

            section { class: "benchmark-section", "aria-labelledby": "web-evidence-title",
                header { class: "benchmark-section-header",
                    div {
                        p { class: "utility-label", "WEB RELEASE + LANDING" }
                        h2 { id: "web-evidence-title", "Browser evidence" }
                    }
                    span { "Logical output · 30-run LCP medians" }
                }
                div { class: "table-scroll", tabindex: "0", "aria-label": "Scrollable browser benchmark comparison",
                    table { class: "evidence-table",
                        thead {
                            tr {
                                th { scope: "col", "Measure" }
                                th { scope: "col", "React Native web" }
                                th { scope: "col", "Dioxus" }
                                th { scope: "col", "Local result" }
                            }
                        }
                        tbody {
                            tr {
                                th { scope: "row", "Logical release output · raw" }
                                td { class: "evidence-number", "17,051,891 B" }
                                td { class: "evidence-number", "2,529,822 B" }
                                td { span { class: "result result-dioxus", "Dioxus smaller" } }
                            }
                            tr {
                                th { scope: "row", "Logical release output · Brotli estimate" }
                                td { class: "evidence-number", "12,390,855 B" }
                                td { class: "evidence-number", "2,222,940 B" }
                                td { span { class: "result result-dioxus", "Dioxus smaller" } }
                            }
                            tr {
                                th { scope: "row", "Landing LCP · cold median" }
                                td { class: "evidence-number", "748 ms" }
                                td { class: "evidence-number", "156 ms" }
                                td { span { class: "result result-dioxus", "Dioxus lower" } }
                            }
                            tr {
                                th { scope: "row", "Landing LCP · warm median" }
                                td { class: "evidence-number", "368 ms" }
                                td { class: "evidence-number", "136 ms" }
                                td { span { class: "result result-dioxus", "Dioxus lower" } }
                            }
                        }
                    }
                }
            }

            section { class: "benchmark-section benchmark-section-react", "aria-labelledby": "worker-evidence-title",
                header { class: "benchmark-section-header",
                    div {
                        p { class: "utility-label", "LOCAL HEALTH ENDPOINT" }
                        h2 { id: "worker-evidence-title", "Backend evidence" }
                    }
                    span { "Local microbenchmark · approximate" }
                }
                div { class: "table-scroll", tabindex: "0", "aria-label": "Scrollable Worker benchmark comparison",
                    table { class: "evidence-table",
                        thead {
                            tr {
                                th { scope: "col", "Measure" }
                                th { scope: "col", "Hono baseline" }
                                th { scope: "col", "workers-rs POC" }
                                th { scope: "col", "Local result" }
                            }
                        }
                        tbody {
                            tr {
                                th { scope: "row", "Health throughput" }
                                td { class: "evidence-number", "1,399.1 req/s" }
                                td { class: "evidence-number", "126.7 req/s" }
                                td { span { class: "result result-react", "Hono higher" } }
                            }
                            tr {
                                th { scope: "row", "Health p95 latency" }
                                td { class: "evidence-number", "25.2 ms" }
                                td { class: "evidence-number", "261.7 ms" }
                                td { span { class: "result result-react", "Hono lower" } }
                            }
                        }
                    }
                }
            }

            section { class: "benchmark-limitations", "aria-labelledby": "limitations-title",
                p { class: "utility-label", "READ BEFORE DECIDING" }
                h2 { id: "limitations-title", "What these numbers do not prove" }
                ul {
                    li { "These are local measurements, not production Cloudflare latency or a global user sample." }
                    li { "The LCP figures are 30-run landing-page medians; they do not establish full feature parity, p75 Core Web Vitals, or native mobile performance." }
                    li { "Release-size totals are logical output and file-by-file Brotli estimates, not bytes transferred for every user journey." }
                    li { "The health test is a narrow endpoint microbenchmark. It exposes a real workers-rs regression, but does not model cached reads, durable messaging, or application-level work." }
                    li { "Authenticated production identity, live UI data adapters, 500-client load, soak behavior, and Android/iOS distributions remain release gates." }
                }
            }
            pre { class: "command-block", code { "# Reproduce against pinned commits and release builds\ndx build --web --release\n# Preserve raw samples, machine details, and browser/tool versions with the report" } }
        }
    }
}

/// Gives first-time Rust contributors an approachable map of this UI slice.
#[component]
pub(crate) fn Docs() -> Element {
    rsx! {
        title { "Contributor docs — Janata Rust POC" }
        div { class: "page docs-page",
            PageHeader {
                kicker: "CONTRIBUTOR DOCS",
                title: "New to Rust? Start with the shape of the app.",
                summary: "You do not need to memorize Rust before contributing. Learn the route you are changing, follow its data boundary, and let the compiler point to the next concrete step.",
                status: "Hosted guide + API reference",
            }
            div { class: "docs-destinations", role: "group", "aria-label": "Hosted documentation",
                a { class: "button button-primary", href: "/docs/guide/", "Open contributor guide" }
                a { class: "button button-quiet", href: "/docs/api/", "Open Rust API reference" }
                p { "These are normal same-origin links so the hosted mdBook and rustdoc sites load outside the Dioxus router." }
            }
            nav { class: "docs-toc", "aria-label": "On this page",
                a { href: "#mental-model", "Mental model" }
                a { href: "#make-change", "Make a change" }
                a { href: "#platforms", "Platforms" }
                a { href: "#boundaries", "POC boundaries" }
            }
            section { id: "mental-model", class: "docs-section",
                p { class: "utility-label", "01 · MENTAL MODEL" }
                h2 { "A route is a screen; a component is a reusable piece." }
                p { "The `Route` enum in `src/main.rs` is the app map. Routed components are re-exported from `src/pages/mod.rs` and implemented in named route modules such as `src/pages/discover.rs`. Dioxus renders components from `rsx!` markup, and signals hold small pieces of interactive state." }
                pre { class: "command-block", code { "Route::Discover (/explore)  →  pages::discover::Discover  →  api::fetch_discover  →  Rust Worker\n        navigation path                presentation             typed request          backend" } }
            }
            section { id: "make-change", class: "docs-section",
                p { class: "utility-label", "02 · MAKE A CHANGE" }
                h2 { "Follow one narrow loop." }
                ol { class: "contributor-steps",
                    li { strong { "Find the route." } span { "Open `src/pages/mod.rs`, then follow the screen re-export to its named route module, such as `src/pages/discover.rs`." } }
                    li { strong { "Change one behavior." } span { "Keep network access behind a contract; do not hide a request inside presentation code." } }
                    li { strong { "Let the tools explain." } span { "Run format, check, tests, and Clippy. Read the first compiler error before the rest." } }
                    li { strong { "Check both shapes." } span { "Review a narrow mobile viewport and a wide desktop viewport, then use the mobile feature compile gate." } }
                }
            }
            section { id: "platforms", class: "docs-section",
                p { class: "utility-label", "03 · PLATFORMS" }
                h2 { "Shared does not mean identical." }
                p { "Web uses real URLs and a desktop navigation rail. Narrow web and native builds use the bottom navigation and single-column layouts. Platform services such as notifications or secure storage belong behind small interfaces so each target can use its native implementation." }
                div { class: "checks",
                    CheckRow { label: "Web feature", detail: "cargo check -p janata-ui --no-default-features --features web", state: "Gate" }
                    CheckRow { label: "Mobile feature", detail: "cargo check -p janata-ui --no-default-features --features mobile", state: "Gate" }
                    CheckRow { label: "WASM target", detail: "cargo check -p janata-ui --target wasm32-unknown-unknown", state: "Gate" }
                }
            }
            section { id: "boundaries", class: "docs-section docs-warning",
                p { class: "utility-label", "04 · POC BOUNDARIES" }
                h2 { "The foundation is real; the member experience is not live yet." }
                p { "Typed read-only Worker APIs and private SpacetimeDB reducers have been proven locally. The community cards on these screens still use small compile-time arrays. Authenticated production identity, authorization, live Worker reads, realtime UI subscriptions, native permissions, and end-to-end mobile distributions remain gated before beta." }
            }
        }
    }
}

/// Gives invalid deep links an actionable way back into the shell.
#[component]
pub(crate) fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        title { "Page not found — Janata Rust POC" }
        div { class: "page not-found",
            p { class: "kicker", "404 · ROUTE NOT FOUND" }
            h1 { "This path is not part of the proof." }
            p { "No screen is registered for /{path}. Use the typed navigation to return to a working route." }
            Link { class: "button button-primary", to: Route::Home, "Return home" }
        }
    }
}
