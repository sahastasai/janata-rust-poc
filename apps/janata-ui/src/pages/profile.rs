use crate::components::{PageHeader, PreviewNotice};
use dioxus::prelude::*;

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
