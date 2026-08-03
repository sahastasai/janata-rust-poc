use crate::{
    Route,
    api::{self, ApiClientError},
    components::PageHeader,
    pages::date_parts,
};
use dioxus::prelude::*;
use janata_api_contract::CenterDetailResponse;
use janata_domain::Event;

/// Public center detail that supports direct web and native deep links.
#[component]
pub(crate) fn CenterDetail(id: String) -> Element {
    let detail = use_resource(use_reactive!(|(id,)| async move {
        api::fetch_center_detail(&id).await
    }));

    rsx! {
        title { "Center — Janata Rust POC" }
        div { class: "page profile-page",
            Link { class: "button button-quiet", to: Route::Discover, "← Back to Discover" }
            match &*detail.read_unchecked() {
                None => loading_state("Loading center details…"),
                Some(Err(error)) => unavailable_state("Center", error),
                Some(Ok(response)) => center_content(response),
            }
        }
    }
}

fn loading_state(message: &str) -> Element {
    rsx! {
        section { class: "detail-section", role: "status", "aria-live": "polite",
            p { class: "utility-label", "PUBLIC CENTER" }
            h1 { "{message}" }
            p { class: "page-summary", "The Rust Worker request is in progress." }
        }
    }
}

fn unavailable_state(kind: &str, error: &ApiClientError) -> Element {
    let heading = if matches!(
        error,
        ApiClientError::NotFound(_) | ApiClientError::InvalidId(_)
    ) {
        format!("{kind} not found")
    } else {
        format!("{kind} unavailable")
    };
    rsx! {
        section { class: "detail-section", role: "alert",
            p { class: "utility-label", "PUBLIC CENTER" }
            h1 { "{heading}" }
            p { class: "page-summary", "{error}" }
            Link { class: "button button-primary", to: Route::Discover, "Browse public centers" }
        }
    }
}

fn center_content(response: &CenterDetailResponse) -> Element {
    let center = &response.center;
    let verification = if center.verified {
        "Verified public listing"
    } else {
        "Public listing"
    };

    rsx! {
        PageHeader {
            kicker: "PUBLIC CENTER",
            title: center.name.clone(),
            summary: center.description.clone().unwrap_or_else(|| "Public center details and upcoming gatherings.".to_owned()),
            status: verification,
        }
        div { class: "profile-grid",
            section { class: "identity-card", "aria-labelledby": "center-about-heading",
                div { class: "center-mark", "CM" }
                h2 { id: "center-about-heading", "About this center" }
                p { class: "role-line", "{center.address}" }
                p { class: "bio",
                    {center.description.as_deref().unwrap_or("No public description has been provided.")}
                }
                div { class: "interest-list",
                    span { "{center.member_count} members" }
                    if center.verified { span { "Verified" } }
                }
            }
            div { class: "profile-details",
                section { class: "detail-section", "aria-labelledby": "center-details-heading",
                    p { class: "utility-label", "DETAILS" }
                    h2 { id: "center-details-heading", "Visit or get in touch" }
                    div { class: "center-row",
                        span { class: "center-mark", "A" }
                        div {
                            h3 { "Address" }
                            p { "{center.address}" }
                        }
                    }
                    if let Some(website) = center.website.as_deref().filter(|url| url.starts_with("https://")) {
                        div { class: "center-row",
                            span { class: "center-mark", "W" }
                            div {
                                h3 { "Website" }
                                p { "{website}" }
                            }
                            a { class: "button button-quiet", href: "{website}", target: "_blank", rel: "noopener noreferrer", "Open site" }
                        }
                    }
                    if let Some(phone) = center.phone.as_deref() {
                        div { class: "center-row",
                            span { class: "center-mark", "P" }
                            div {
                                h3 { "Phone" }
                                p { "{phone}" }
                            }
                        }
                    }
                    if let Some(acharya) = center.acharya.as_deref() {
                        div { class: "center-row",
                            span { class: "center-mark", "ॐ" }
                            div {
                                h3 { "Resident Acharya" }
                                p { "{acharya}" }
                            }
                        }
                    }
                    if let Some(contact) = center.point_of_contact.as_deref() {
                        div { class: "center-row",
                            span { class: "center-mark", "C" }
                            div {
                                h3 { "Point of contact" }
                                p { "{contact}" }
                            }
                        }
                    }
                }
            }
        }

        section { class: "detail-section", "aria-labelledby": "center-events-heading",
            p { class: "utility-label", "UPCOMING" }
            h2 { id: "center-events-heading", "Events at this center" }
            if response.events.items.is_empty() {
                p { class: "page-summary", role: "status", "No upcoming public events are listed for this center." }
            } else {
                for event in &response.events.items {
                    {event_row(event)}
                }
            }
        }

        section { class: "detail-section", "aria-labelledby": "center-board-heading",
            p { class: "utility-label", "BOARD" }
            h2 { id: "center-board-heading", "Center conversation" }
            p { class: "page-summary", "Public visitors can browse this center. Posting and replies require verified membership and are intentionally unavailable in this read-only proof of concept." }
        }
    }
}

fn event_row(event: &Event) -> Element {
    let (month, day) = date_parts(&event.date);
    rsx! {
        Link { class: "compact-event", to: Route::EventDetail { id: event.id.to_string() },
            time { datetime: "{event.date}", "{month} {day}" }
            div {
                h3 { "{event.title}" }
                p { "{event.time_label} · {event.attendee_count} attending" }
            }
        }
    }
}
