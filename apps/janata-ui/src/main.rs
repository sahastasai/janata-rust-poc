//! Cross-platform entry point for the Janata Dioxus proof of concept.
//!
//! This binary contains presentation and local interaction state. The integrated
//! POC also has typed, read-only Worker contracts and locally proven private
//! reducers; authenticated production identity and live UI adapters remain
//! explicit gates rather than simulated behavior.

mod components;
mod model;
mod pages;

use components::AppShell;
use dioxus::prelude::*;
use pages::{Benchmarks, Connect, Discover, Docs, Feed, Home, NotFound, Profile};

static APP_CSS: Asset = asset!("/assets/main.css");
static INCLUSIVE_REGULAR: Asset = asset!("/assets/fonts/InclusiveSans-Regular.ttf");
static INCLUSIVE_SEMIBOLD: Asset = asset!("/assets/fonts/InclusiveSans-SemiBold.ttf");
static INTER_REGULAR: Asset = asset!("/assets/fonts/Inter-Regular.ttf");
static INTER_SEMIBOLD: Asset = asset!("/assets/fonts/Inter-SemiBold.ttf");

fn main() {
    dioxus::launch(App);
}

/// Renders shared document assets and the type-safe application router.
#[component]
fn App() -> Element {
    // Referencing font assets from generated CSS lets the bundler fingerprint
    // them while preserving a single semantic font stack across platforms.
    let font_faces = format!(
        r#"
        @font-face {{ font-family: 'Inclusive Sans'; src: url('{INCLUSIVE_REGULAR}') format('truetype'); font-display: swap; font-style: normal; font-weight: 400; }}
        @font-face {{ font-family: 'Inclusive Sans'; src: url('{INCLUSIVE_SEMIBOLD}') format('truetype'); font-display: swap; font-style: normal; font-weight: 600; }}
        @font-face {{ font-family: 'Inter'; src: url('{INTER_REGULAR}') format('truetype'); font-display: swap; font-style: normal; font-weight: 400; }}
        @font-face {{ font-family: 'Inter'; src: url('{INTER_SEMIBOLD}') format('truetype'); font-display: swap; font-style: normal; font-weight: 600; }}
        "#,
    );

    rsx! {
        link { rel: "stylesheet", href: APP_CSS }
        style { {font_faces} }
        meta { name: "theme-color", content: "#F7F8F4" }
        meta {
            name: "description",
            content: "A cross-platform Rust proof of concept for Chinmaya Janata.",
        }
        Router::<Route> {}
    }
}

/// Every reviewable screen in the UI spike.
///
/// Keeping destinations in one enum gives web deep links and native navigation
/// the same compile-time checked source of truth.
#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Home,
        #[route("/discover")]
        Discover,
        #[route("/feed")]
        Feed,
        #[route("/connect")]
        Connect,
        #[route("/profile")]
        Profile,
        #[route("/benchmarks")]
        Benchmarks,
        #[route("/docs")]
        Docs,
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::Route;

    #[test]
    fn primary_routes_have_stable_paths() {
        let routes = [
            (Route::Home, "/"),
            (Route::Discover, "/discover"),
            (Route::Feed, "/feed"),
            (Route::Connect, "/connect"),
            (Route::Profile, "/profile"),
            (Route::Benchmarks, "/benchmarks"),
            (Route::Docs, "/docs"),
        ];

        for (route, expected) in routes {
            assert_eq!(route.to_string(), expected);
        }
    }
}
