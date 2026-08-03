//! Shared navigation and small presentation primitives.

use crate::Route;
use dioxus::prelude::*;

/// Provides desktop, mobile, and native-friendly navigation around each route.
#[component]
pub(crate) fn AppShell() -> Element {
    rsx! {
        div { class: "app-shell",
            a { class: "skip-link", href: "#main-content", "Skip to content" }

            aside { class: "sidebar", "aria-label": "Primary navigation",
                Brand {}
                div { class: "thread-line", "aria-hidden": "true" }
                nav { class: "primary-nav",
                    NavLink { route: Route::Home, glyph: "H", label: "Home" }
                    NavLink { route: Route::Discover, glyph: "D", label: "Discover" }
                    NavLink { route: Route::Feed, glyph: "F", label: "Feed" }
                    NavLink { route: Route::Connect, glyph: "C", label: "Connect" }
                    NavLink { route: Route::Profile, glyph: "P", label: "Profile" }
                }
                nav { class: "evidence-nav", "aria-label": "Proof of concept evidence",
                    NavLink { route: Route::Benchmarks, glyph: "B", label: "Benchmarks" }
                    NavLink { route: Route::Docs, glyph: "?", label: "Contributor docs" }
                }
                div { class: "sidebar-note",
                    span { class: "status-dot", "aria-hidden": "true" }
                    div {
                        strong { "Integrated Rust POC" }
                        small { "Read-only · identity gated" }
                    }
                }
            }

            header { class: "mobile-header",
                Brand {}
                span { class: "poc-chip", "Rust POC" }
            }

            main { id: "main-content", class: "main-content", tabindex: "-1",
                Outlet::<Route> {}
            }

            nav { class: "bottom-nav", "aria-label": "Mobile navigation",
                NavLink { route: Route::Home, glyph: "H", label: "Home" }
                NavLink { route: Route::Discover, glyph: "D", label: "Discover" }
                NavLink { route: Route::Feed, glyph: "F", label: "Feed" }
                NavLink { route: Route::Connect, glyph: "C", label: "Connect" }
                NavLink { route: Route::Profile, glyph: "P", label: "Profile" }
            }
        }
    }
}

#[component]
fn Brand() -> Element {
    rsx! {
        Link { class: "brand", to: Route::Home,
            span { class: "brand-mark", "J" }
            span {
                strong { "JANATA" }
                small { "Chinmaya community" }
            }
        }
    }
}

#[component]
fn NavLink(route: Route, glyph: String, label: String) -> Element {
    rsx! {
        Link {
            class: "nav-link",
            active_class: "is-active",
            to: route,
            span { class: "nav-glyph", "{glyph}" }
            span { class: "nav-label", "{label}" }
        }
    }
}

/// Starts a content screen with consistent context and status language.
#[component]
pub(crate) fn PageHeader(
    kicker: String,
    title: String,
    summary: String,
    #[props(default)] status: Option<String>,
) -> Element {
    rsx! {
        header { class: "page-header",
            div {
                p { class: "kicker", "{kicker}" }
                h1 { "{title}" }
                p { class: "page-summary", "{summary}" }
            }
            if let Some(status) = status {
                span { class: "preview-chip", "{status}" }
            }
        }
    }
}

/// Marks content that exists only to exercise layout and interaction behavior.
#[component]
pub(crate) fn PreviewNotice() -> Element {
    rsx! {
        div { class: "preview-notice", role: "note",
            span { class: "preview-icon", "P" }
            p {
                strong { "Preview data" }
                " — realistic static content for UI review. It is not fetched, saved, or sent."
            }
        }
    }
}

/// Renders a compact, semantic status row used in evidence and docs screens.
#[component]
pub(crate) fn CheckRow(label: String, detail: String, state: String) -> Element {
    rsx! {
        div { class: "check-row",
            div {
                strong { "{label}" }
                span { "{detail}" }
            }
            span { class: "check-state", "{state}" }
        }
    }
}
