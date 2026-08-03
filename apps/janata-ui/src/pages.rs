//! Routed screens for the integrated Janata proof of concept.

use crate::{
    Route,
    components::{CheckRow, PageHeader, PreviewNotice},
    model::{CONVERSATIONS, EVENTS, EventPreview, POSTS, PostPreview},
};
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

/// Lets reviewers filter and inspect the event discovery layout locally.
#[component]
pub(crate) fn Discover() -> Element {
    let mut selected = use_signal(|| String::from("All"));
    let filters = ["All", "Satsang", "CHYK", "Seva", "Vedanta"];

    rsx! {
        title { "Discover — Janata Rust POC" }
        div { class: "page",
            PageHeader {
                kicker: "DISCOVER",
                title: "Find what brings you together",
                summary: "Browse gatherings across centers, then narrow the view by the kind of practice or service you want.",
                status: "Local filter",
            }
            PreviewNotice {}
            div { class: "toolbar",
                div { class: "segmented", role: "group", "aria-label": "Filter events by category",
                    for filter in filters {
                        button {
                            class: if selected() == filter { "segment is-selected" } else { "segment" },
                            r#type: "button",
                            aria_pressed: if selected() == filter { "true" } else { "false" },
                            onclick: move |_| selected.set(filter.to_string()),
                            "{filter}"
                        }
                    }
                }
                button { class: "icon-button", r#type: "button", "aria-label": "Map view is not connected in this proof of concept", "Map view · soon" }
            }
            section { class: "event-grid", "aria-label": "Preview events",
                for event in EVENTS.iter().filter(|event| selected() == "All" || selected() == event.category) {
                    {event_card(event)}
                }
            }
        }
    }
}

fn event_card(event: &EventPreview) -> Element {
    rsx! {
        article { class: "event-card",
            div { class: "date-block",
                span { "{event.month}" }
                strong { "{event.day}" }
            }
            div { class: "event-body",
                span { class: "category-chip", "{event.category}" }
                h2 { "{event.title}" }
                p { "{event.center}" }
                small { "{event.place}" }
            }
            button { class: "arrow-button", r#type: "button", "aria-label": "Event details are not connected in this proof of concept", "→" }
        }
    }
}

/// Exercises a readable, scoped community feed without claiming live content.
#[component]
pub(crate) fn Feed() -> Element {
    let mut selected = use_signal(|| String::from("Your circles"));

    rsx! {
        title { "Feed — Janata Rust POC" }
        div { class: "page reading-page",
            PageHeader {
                kicker: "FEED",
                title: "What your circles are sharing",
                summary: "Center updates, event threads, and community notes stay close to the people and places that give them context.",
                status: "Preview data",
            }
            div { class: "feed-layout",
                aside { class: "feed-scope", "aria-label": "Feed scope",
                    p { class: "utility-label", "SHOW POSTS FROM" }
                    for scope in ["Your circles", "Your center", "Community"] {
                        button {
                            class: if selected() == scope { "scope-button is-selected" } else { "scope-button" },
                            r#type: "button",
                            onclick: move |_| selected.set(scope.to_string()),
                            span { "{scope}" }
                            if selected() == scope { span { "✓" } }
                        }
                    }
                    p { class: "scope-help", "This control changes presentation state only." }
                }
                div { class: "feed-column",
                    PreviewNotice {}
                    div { class: "scope-result", "Showing: {selected}" }
                    for post in POSTS {
                        {post_card(&post)}
                    }
                }
            }
        }
    }
}

fn post_card(post: &PostPreview) -> Element {
    rsx! {
        article { class: "post-card",
            header {
                span { class: "avatar", "{post.initials}" }
                div {
                    h2 { "{post.author}" }
                    p { "{post.source} · {post.time}" }
                }
                button { class: "menu-button", r#type: "button", "aria-label": "Post menu is not connected", "•••" }
            }
            p { class: "post-body", "{post.body}" }
            footer {
                button { r#type: "button", "aria-label": "Reactions are not connected", "Appreciate" }
                button { r#type: "button", "aria-label": "Replies are not connected", "{post.replies} replies" }
            }
        }
    }
}

/// Demonstrates the message list/detail composition shared by web and mobile.
#[component]
pub(crate) fn Connect() -> Element {
    let mut selected = use_signal(|| 0_usize);
    let active = CONVERSATIONS[selected()];

    rsx! {
        title { "Connect — Janata Rust POC" }
        div { class: "page connect-page",
            PageHeader {
                kicker: "CONNECT",
                title: "Conversations with context",
                summary: "Keep direct messages, event chats, and center coordination in one calm workspace.",
                status: "Private reducers proven locally",
            }
            PreviewNotice {}
            section { class: "messenger", "aria-label": "Message preview",
                div { class: "thread-list",
                    label { class: "search-field",
                        span { class: "sr-only", "Search conversations" }
                        input { r#type: "search", placeholder: "Search conversations", disabled: true }
                    }
                    for (index, thread) in CONVERSATIONS.iter().enumerate() {
                        button {
                            class: if selected() == index { "thread-button is-selected" } else { "thread-button" },
                            r#type: "button",
                            onclick: move |_| selected.set(index),
                            span { class: "avatar", "{thread.initials}" }
                            span { class: "thread-copy",
                                strong { "{thread.name}" }
                                small { "{thread.preview}" }
                            }
                            time { "{thread.time}" }
                        }
                    }
                }
                div { class: "chat-panel",
                    header { class: "chat-header",
                        span { class: "avatar avatar-large", "{active.initials}" }
                        div {
                            h2 { "{active.name}" }
                            p { "{active.context}" }
                        }
                    }
                    div { class: "message-space",
                        p { class: "integration-label", "LOCAL LAYOUT PREVIEW" }
                        div { class: "message-bubble message-other", "Can everyone see the event notes?" }
                        div { class: "message-bubble message-self", "Yes — I have them open. The summary is clear." }
                        p { class: "message-time", "Messages shown here are static preview content." }
                    }
                    form { class: "composer", onsubmit: move |event| event.prevent_default(),
                        label { class: "sr-only", r#for: "message-draft", "Message" }
                        input { id: "message-draft", placeholder: "Live UI subscription is gated", disabled: true }
                        button { r#type: "submit", disabled: true, "Send" }
                    }
                }
            }
        }
    }
}

/// Presents the member profile information architecture with explicit samples.
#[component]
pub(crate) fn Profile() -> Element {
    rsx! {
        title { "Profile — Janata Rust POC" }
        div { class: "page profile-page",
            PageHeader {
                kicker: "PROFILE",
                title: "Your place in the community",
                summary: "Identity, centers, gatherings, and interests are grouped for quick scanning without turning participation into a leaderboard.",
                status: "Preview data",
            }
            PreviewNotice {}
            div { class: "profile-grid",
                section { class: "identity-card",
                    div { class: "profile-avatar", "AS" }
                    p { class: "utility-label", "SAMPLE MEMBER" }
                    h2 { "Asha Srinivasan" }
                    p { class: "role-line", "Verified member · San Jose" }
                    p { class: "bio", "Interested in Vedanta study, youth programs, and making community events easier to find." }
                    div { class: "interest-list",
                        span { "Vedanta" }
                        span { "CHYK" }
                        span { "Seva" }
                    }
                    div { class: "profile-actions",
                        button { class: "button button-dark", r#type: "button", disabled: true, "Edit after sign-in" }
                        button { class: "button button-quiet", r#type: "button", disabled: true, "Share after sign-in" }
                    }
                }
                div { class: "profile-details",
                    section { class: "detail-section",
                        p { class: "utility-label", "YOUR CENTERS" }
                        article { class: "center-row",
                            span { class: "center-mark", "SJ" }
                            div {
                                h3 { "Chinmaya Mission San Jose" }
                                p { "San Jose, California" }
                            }
                            span { "Home" }
                        }
                    }
                    section { class: "detail-section",
                        p { class: "utility-label", "UPCOMING" }
                        article { class: "compact-event",
                            time { "AUG 09" }
                            div {
                                h3 { "Sunday Satsang & Family Lunch" }
                                p { "10:00 AM · Main hall" }
                            }
                        }
                        article { class: "compact-event",
                            time { "AUG 21" }
                            div {
                                h3 { "Community Pantry" }
                                p { "8:30 AM · Volunteer entrance" }
                            }
                        }
                    }
                }
            }
        }
    }
}

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
                p { "The `Route` enum in `src/main.rs` is the app map. Each enum variant has a component with the same name in `src/pages.rs`. Dioxus renders components from `rsx!` markup, and signals hold small pieces of interactive state." }
                pre { class: "command-block", code { "Route::Discover  →  pages::Discover  →  EventPreview\n        navigation     presentation       preview model" } }
            }
            section { id: "make-change", class: "docs-section",
                p { class: "utility-label", "02 · MAKE A CHANGE" }
                h2 { "Follow one narrow loop." }
                ol { class: "contributor-steps",
                    li { strong { "Find the route." } span { "Open `src/pages.rs` and locate the screen name." } }
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
