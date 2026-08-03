use crate::{
    components::{PageHeader, PreviewNotice},
    model::{EVENTS, EventPreview},
};
use dioxus::prelude::*;

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
