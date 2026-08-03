use crate::{
    Route,
    api::{self, ApiClientError},
    components::PageHeader,
    pages::category_label,
};
use dioxus::prelude::*;
use janata_api_contract::EventDetailResponse;

/// Public event detail that never implies guest registration has occurred.
#[component]
pub(crate) fn EventDetail(id: String) -> Element {
    let detail = use_resource(use_reactive!(|(id,)| async move {
        api::fetch_event_detail(&id).await
    }));

    rsx! {
        title { "Event — Janata Rust POC" }
        div { class: "page profile-page",
            Link { class: "button button-quiet", to: Route::Discover, "← Back to Discover" }
            match &*detail.read_unchecked() {
                None => loading_state(),
                Some(Err(error)) => unavailable_state(error),
                Some(Ok(response)) => event_content(response),
            }
        }
    }
}

fn loading_state() -> Element {
    rsx! {
        section { class: "detail-section", role: "status", "aria-live": "polite",
            p { class: "utility-label", "PUBLIC EVENT" }
            h1 { "Loading event details…" }
            p { class: "page-summary", "The Rust Worker request is in progress." }
        }
    }
}

fn unavailable_state(error: &ApiClientError) -> Element {
    let heading = if matches!(
        error,
        ApiClientError::NotFound(_) | ApiClientError::InvalidId(_)
    ) {
        "Event not found"
    } else {
        "Event unavailable"
    };
    rsx! {
        section { class: "detail-section", role: "alert",
            p { class: "utility-label", "PUBLIC EVENT" }
            h1 { "{heading}" }
            p { class: "page-summary", "{error}" }
            Link { class: "button button-primary", to: Route::Discover, "Browse public events" }
        }
    }
}

fn event_content(response: &EventDetailResponse) -> Element {
    let event = &response.event;
    let category = category_label(event.category.as_deref());
    let status = if event.official {
        "Official public event"
    } else {
        "Public event"
    };

    rsx! {
        PageHeader {
            kicker: "PUBLIC EVENT",
            title: event.title.clone(),
            summary: event.description.clone(),
            status,
        }
        div { class: "profile-grid",
            section { class: "identity-card", "aria-labelledby": "event-summary-heading",
                span { class: "category-chip", "{category}" }
                h2 { id: "event-summary-heading", "{event.date}" }
                p { class: "role-line", "{event.time_label}" }
                p { class: "bio", "{event.location}" }
                div { class: "interest-list",
                    span { "{event.attendee_count} attending" }
                    if event.recurring { span { "Recurring" } }
                    if event.requires_verified { span { "Verified account required" } }
                }
                if let Some(center_id) = event.center_id {
                    div { class: "profile-actions",
                        Link { class: "button button-quiet", to: Route::CenterDetail { id: center_id.to_string() }, "View hosting center" }
                    }
                }
            }
            div { class: "profile-details",
                section { class: "detail-section", "aria-labelledby": "event-details-heading",
                    p { class: "utility-label", "DETAILS" }
                    h2 { id: "event-details-heading", "Plan your visit" }
                    div { class: "center-row",
                        span { class: "center-mark", "D" }
                        div {
                            h3 { "Date and time" }
                            p {
                                "{event.date} · {event.time_label}"
                                if let Some(end_date) = event.end_date.as_deref() { " through {end_date}" }
                            }
                        }
                    }
                    div { class: "center-row",
                        span { class: "center-mark", "L" }
                        div {
                            h3 { "Location" }
                            p { "{event.location}" }
                            if let Some(address) = event.address.as_deref() { p { "{address}" } }
                        }
                    }
                    if let Some(contact) = event.point_of_contact.as_deref() {
                        div { class: "center-row",
                            span { class: "center-mark", "C" }
                            div {
                                h3 { "Point of contact" }
                                p { "{contact}" }
                            }
                        }
                    }
                    if let Some(url) = event.external_url.as_deref().filter(|url| url.starts_with("https://")) {
                        div { class: "center-row",
                            span { class: "center-mark", "I" }
                            div {
                                h3 { "Official information" }
                                p { "Open the source page for additional details." }
                            }
                            a { class: "button button-quiet", href: "{url}", target: "_blank", rel: "noopener noreferrer", "Open page" }
                        }
                    }
                }
            }
        }

        section { class: "detail-section", "aria-labelledby": "event-registration-heading",
            p { class: "utility-label", "REGISTRATION" }
            h2 { id: "event-registration-heading", "Attend this gathering" }
            if let Some(url) = event.signup_url.as_deref().filter(|url| url.starts_with("https://")) {
                p { class: "page-summary", "Registration is handled by the organizer on its official site. Janata has not recorded an RSVP." }
                a { class: "button button-primary", href: "{url}", target: "_blank", rel: "noopener noreferrer", "Visit official registration" }
            } else if event.allow_janata_signup {
                p { class: "page-summary", "Janata registration requires authenticated identity and a write-enabled service. Those capabilities are intentionally unavailable in this public, read-only proof of concept." }
                button { class: "button button-primary", r#type: "button", disabled: true, "Janata RSVP unavailable" }
            } else {
                p { class: "page-summary", "The organizer has not published a registration link for this event." }
            }
        }

        section { class: "detail-section", "aria-labelledby": "event-attendees-heading",
            p { class: "utility-label", "ATTENDANCE" }
            h2 { id: "event-attendees-heading", "{event.attendee_count} people attending" }
            p { class: "page-summary", "Only the public aggregate count is shown. Names and contact details are protected and are not part of this endpoint." }
        }

        section { class: "detail-section", "aria-labelledby": "event-comments-heading",
            p { class: "utility-label", "COMMENTS" }
            h2 { id: "event-comments-heading", "Event conversation" }
            p { class: "page-summary", "Conversation access requires an authenticated attendee, creator, or administrator. Public visitors can read event details but cannot post from this proof of concept." }
        }
    }
}
