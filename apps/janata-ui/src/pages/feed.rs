use crate::{
    components::{PageHeader, PreviewNotice},
    model::{POSTS, PostPreview},
};
use dioxus::prelude::*;

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
