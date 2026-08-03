use crate::{
    Route,
    api::{self, ApiClientError},
    components::PageHeader,
    pages::{category_label, date_parts},
};
use dioxus::prelude::*;
use janata_api_contract::DiscoverResponse;
use janata_domain::{Center, CenterId, Event as PublicEvent};

const CATEGORY_FILTERS: &[(&str, &str)] = &[
    ("", "All gatherings"),
    ("meditation", "Meditation"),
    ("satsang", "Satsang"),
    ("seva", "Seva"),
];

/// Loads and filters the public Worker catalog without requiring an account.
#[component]
pub(crate) fn Discover() -> Element {
    let mut selected_category = use_signal(String::new);
    let mut selected_center = use_signal(|| None::<CenterId>);
    let mut discovery = use_resource(move || {
        let category = selected_category();
        let center_id = selected_center();
        async move { api::fetch_discover(Some(category), center_id).await }
    });

    rsx! {
        title { "Discover — Janata Rust POC" }
        div { class: "page",
            PageHeader {
                kicker: "DISCOVER",
                title: "Find what brings you together",
                summary: "Browse public centers and gatherings without signing in. Narrow the live catalog by practice, service, or center.",
                status: "Live Rust API",
            }
            div { class: "preview-notice", role: "note",
                span { class: "preview-icon", "R" }
                p {
                    strong { "Read-only public catalog" }
                    " — loaded from the Rust Worker. Registration and member-only boards are not enabled in this proof of concept."
                }
            }
            div { class: "toolbar",
                div { class: "segmented", role: "group", "aria-label": "Filter events by category",
                    for &(value, label) in CATEGORY_FILTERS {
                        button {
                            class: if selected_category() == value { "segment is-selected" } else { "segment" },
                            r#type: "button",
                            aria_pressed: if selected_category() == value { "true" } else { "false" },
                            onclick: move |_| selected_category.set(value.to_owned()),
                            "{label}"
                        }
                    }
                }
                button {
                    class: "icon-button",
                    r#type: "button",
                    onclick: move |_| discovery.restart(),
                    "Refresh"
                }
            }
            match &*discovery.read_unchecked() {
                None => loading_state(),
                Some(Err(error)) => error_state(error, move |_| discovery.restart()),
                Some(Ok(response)) => discover_content(
                    response,
                    selected_center(),
                    move |center_id| selected_center.set(center_id),
                ),
            }
        }
    }
}

fn loading_state() -> Element {
    rsx! {
        section { class: "detail-section", role: "status", "aria-live": "polite",
            p { class: "utility-label", "Loading public catalog" }
            h2 { "Gathering centers and events…" }
            p { class: "page-summary", "The Worker request is in progress." }
        }
    }
}

fn error_state<F>(error: &ApiClientError, retry: F) -> Element
where
    F: FnMut(Event<MouseData>) + 'static,
{
    rsx! {
        section { class: "detail-section", role: "alert",
            p { class: "utility-label", "Catalog unavailable" }
            h2 { "Discovery could not load" }
            p { class: "page-summary", "{error}" }
            button { class: "button button-quiet", r#type: "button", onclick: retry, "Try again" }
        }
    }
}

fn discover_content<F>(
    response: &DiscoverResponse,
    selected_center: Option<CenterId>,
    select_center: F,
) -> Element
where
    F: FnMut(Option<CenterId>) + Clone + 'static,
{
    let no_events = response.events.is_empty();
    let no_centers = response.centers.is_empty();

    rsx! {
        if !no_centers {
            section { class: "detail-section", "aria-labelledby": "center-filter-heading",
                p { class: "utility-label", "AREA" }
                h2 { id: "center-filter-heading", "Choose a center" }
                div { class: "segmented", role: "group", "aria-label": "Filter gatherings by center",
                    button {
                        class: if selected_center.is_none() { "segment is-selected" } else { "segment" },
                        r#type: "button",
                        aria_pressed: if selected_center.is_none() { "true" } else { "false" },
                        onclick: {
                            let mut select_center = select_center.clone();
                            move |_| select_center(None)
                        },
                        "All centers"
                    }
                    for center in &response.centers {
                        {
                            let center_id = center.id;
                            let mut select_center = select_center.clone();
                            rsx! {
                                button {
                                    class: if selected_center == Some(center_id) { "segment is-selected" } else { "segment" },
                                    r#type: "button",
                                    aria_pressed: if selected_center == Some(center_id) { "true" } else { "false" },
                                    onclick: move |_| select_center(Some(center_id)),
                                    "{center.name}"
                                }
                            }
                        }
                    }
                }
            }
        }

        section { "aria-labelledby": "event-results-heading",
            p { class: "utility-label", "GATHERINGS" }
            h2 { id: "event-results-heading", "Upcoming events" }
            if no_events {
                div { class: "detail-section", role: "status",
                    h3 { "No gatherings match these filters" }
                    p { class: "page-summary", "Choose another category or center to continue browsing." }
                }
            } else {
                div { class: "event-grid",
                    for event in &response.events {
                        {event_card(event, &response.centers)}
                    }
                }
            }
        }

        section { class: "detail-section", "aria-labelledby": "center-results-heading",
            p { class: "utility-label", "CENTERS" }
            h2 { id: "center-results-heading", "Public center directory" }
            if no_centers {
                div { role: "status",
                    h3 { "No centers are available" }
                    p { class: "page-summary", "The public directory returned no center records." }
                }
            } else {
                for center in &response.centers {
                    {center_row(center)}
                }
            }
        }
    }
}

fn event_card(event: &PublicEvent, centers: &[Center]) -> Element {
    let (month, day) = date_parts(&event.date);
    let category = category_label(event.category.as_deref());
    let center_name = event
        .center_id
        .and_then(|id| centers.iter().find(|center| center.id == id))
        .map(|center| center.name.as_str())
        .unwrap_or("Independent gathering");

    rsx! {
        article { class: "event-card",
            div { class: "date-block", aria_label: "{event.date}",
                span { "{month}" }
                strong { "{day}" }
            }
            div { class: "event-body",
                span { class: "category-chip", "{category}" }
                h2 {
                    Link { to: Route::EventDetail { id: event.id.to_string() }, "{event.title}" }
                }
                p { "{center_name} · {event.time_label}" }
                small { "{event.location} · {event.attendee_count} attending" }
            }
            Link {
                class: "arrow-button",
                to: Route::EventDetail { id: event.id.to_string() },
                aria_label: "View details for {event.title}",
                "→"
            }
        }
    }
}

fn center_row(center: &Center) -> Element {
    rsx! {
        div { class: "center-row",
            span { class: "center-mark", "CM" }
            div {
                h3 {
                    Link { to: Route::CenterDetail { id: center.id.to_string() }, "{center.name}" }
                }
                p { "{center.address} · {center.member_count} members" }
            }
            Link {
                class: "button button-quiet",
                to: Route::CenterDetail { id: center.id.to_string() },
                "View center"
            }
        }
    }
}
