use crate::{
    components::{PageHeader, PreviewNotice},
    model::CONVERSATIONS,
};
use dioxus::prelude::*;

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
