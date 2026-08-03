//! Stable Serde contracts shared by the Dioxus UI and Cloudflare Worker.
//!
//! Contracts deliberately use domain types instead of loose JSON maps. This
//! makes breaking changes visible to the compiler and documentation generator.

use janata_domain::{BoardPost, Center, Event, Notification};
use serde::{Deserialize, Serialize};

/// Successful health response returned by `GET /api/health`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// Human-readable service state.
    pub status: String,
    /// POC application version.
    pub version: String,
    /// Git revision associated with the deployed build, when injected.
    pub revision: Option<String>,
}

/// Stable error envelope safe to expose to clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Machine-readable error code.
    pub code: String,
    /// Plain-language resolution-oriented message.
    pub message: String,
    /// Request identifier for support correlation.
    pub request_id: String,
}

/// Stable error wrapper returned for every non-successful API response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    /// The client-safe failure and support correlation identifier.
    pub error: ApiError,
}

/// Paginated response shared by list endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    /// Current page of records.
    pub items: Vec<T>,
    /// Opaque cursor for the next page.
    pub next_cursor: Option<String>,
}

/// Credentials submitted to the browser authentication endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    /// Normalized email address. The server still normalizes and validates it.
    pub login: String,
    /// User-entered password; never logged or persisted in this representation.
    pub password: String,
}

/// Invite presented before an invite-only registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateInviteRequest {
    /// Opaque invite code. The Worker hashes it before querying D1.
    pub code: String,
}

/// Result of checking an invite without consuming it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteValidationResponse {
    /// Whether the code is active, unexpired, and has remaining uses.
    pub valid: bool,
    /// Optional public inviter display name; absent for cohort invites.
    pub inviter_name: Option<String>,
}

/// Invite-gated account registration request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// Email address, normalized again by the server.
    pub email: String,
    /// Public member name used by the authenticated greeting.
    pub display_name: String,
    /// User-entered password; bounded and hashed inside the Worker.
    pub password: String,
    /// Opaque invite code consumed atomically with account creation.
    pub invite_code: String,
}

/// Successful account registration response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    /// True only after the user and invite-use updates commit.
    pub registered: bool,
    /// Normalized email associated with the new account.
    pub email: String,
}

/// Minimal authenticated identity safe to return to the browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    /// Immutable account identifier.
    pub id: String,
    /// Normalized account email.
    pub email: String,
    /// Public display name.
    pub display_name: String,
    /// Explicit verification tier carried by the account.
    pub verification_level: i32,
    /// Named role; authorization must still check capabilities server-side.
    pub role: String,
}

/// Browser session summary. Opaque session and CSRF tokens are never JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    /// True when the HttpOnly cookie resolves to an active D1 session.
    pub authenticated: bool,
    /// Authenticated member identity, absent for guests.
    pub user: Option<SessionUser>,
}

/// Successful logout response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutResponse {
    /// True after the current session row has been revoked.
    pub logged_out: bool,
}

/// Combined discovery payload used by responsive list/map experiences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverResponse {
    /// Centers visible under the active filter.
    pub centers: Vec<Center>,
    /// Events visible under the active filter.
    pub events: Vec<Event>,
}

/// Public, cursor-paginated center collection.
pub type CentersResponse = Page<Center>;

/// Public, cursor-paginated event collection.
pub type EventsResponse = Page<Event>;

/// One public center plus its bounded upcoming-event page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CenterDetailResponse {
    /// Requested center.
    pub center: Center,
    /// Upcoming events owned by this center.
    pub events: Page<Event>,
}

/// One public event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDetailResponse {
    /// Requested event.
    pub event: Event,
}

/// Community feed response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedResponse {
    /// Board posts the current member may read.
    pub posts: Page<BoardPost>,
}

/// Notification center response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationsResponse {
    /// Notifications ordered newest-first.
    pub notifications: Page<Notification>,
    /// Unread total at response time.
    pub unread_count: u32,
}

/// A documented API surface item used by generated reference pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointSpec {
    /// HTTP method.
    pub method: &'static str,
    /// Route path.
    pub path: &'static str,
    /// Required authorization boundary.
    pub authorization: &'static str,
    /// Short behavior summary.
    pub summary: &'static str,
}

/// Typed public endpoints currently implemented by the Rust Worker.
pub const FOUNDATION_ENDPOINTS: &[EndpointSpec] = &[
    EndpointSpec {
        method: "GET",
        path: "/api/health",
        authorization: "public",
        summary: "Report POC service and revision status",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/discover",
        authorization: "public",
        summary: "Read a bounded center and event discovery snapshot",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/centers",
        authorization: "public",
        summary: "List public centers with cursor pagination",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/centers/:id",
        authorization: "public",
        summary: "Read one center and its bounded upcoming events",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/events",
        authorization: "public",
        summary: "List and filter public events with cursor pagination",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/events/:id",
        authorization: "public",
        summary: "Read one public event",
    },
    EndpointSpec {
        method: "POST",
        path: "/api/v1/auth/invite/validate",
        authorization: "public + exact browser origin",
        summary: "Validate an opaque invite without consuming it",
    },
    EndpointSpec {
        method: "POST",
        path: "/api/v1/auth/register",
        authorization: "public + exact browser origin",
        summary: "Create an invite-gated local POC account",
    },
    EndpointSpec {
        method: "POST",
        path: "/api/v1/auth/login",
        authorization: "public + exact browser origin",
        summary: "Create a revocable HttpOnly web session",
    },
    EndpointSpec {
        method: "GET",
        path: "/api/v1/auth/session",
        authorization: "optional HttpOnly web session",
        summary: "Resolve the current guest or member session",
    },
    EndpointSpec {
        method: "POST",
        path: "/api/v1/auth/logout",
        authorization: "web session + exact origin + CSRF",
        summary: "Revoke the current web session and clear cookies",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_contract_uses_camel_case() {
        let response = HealthResponse {
            status: "ok".to_owned(),
            version: "0.1.0".to_owned(),
            revision: Some("abc123".to_owned()),
        };
        let value = serde_json::to_value(response).expect("health response should serialize");
        assert_eq!(value["revision"], "abc123");
        assert!(value.get("request_id").is_none());
    }

    #[test]
    fn error_envelope_preserves_the_support_identifier() {
        let response = ApiErrorEnvelope {
            error: ApiError {
                code: "not_found".to_owned(),
                message: "Event not found.".to_owned(),
                request_id: "request-123".to_owned(),
            },
        };
        let value = serde_json::to_value(response).expect("error response should serialize");
        assert_eq!(value["error"]["requestId"], "request-123");
        assert!(value["error"].get("request_id").is_none());
    }

    #[test]
    fn public_detail_contracts_round_trip() {
        let center: Center = serde_json::from_value(serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000065",
            "name": "Chinmaya Vrindavan",
            "address": "San Jose, California",
            "latitude": 37.3382,
            "longitude": -121.8863,
            "website": "https://www.cmsj.org/",
            "imageUrl": null,
            "phone": null,
            "acharya": "Swami Shantananda",
            "pointOfContact": "Center office",
            "description": "A public sample center.",
            "memberCount": 108,
            "verified": true
        }))
        .expect("center contract should deserialize");
        let response = CenterDetailResponse {
            center,
            events: Page {
                items: Vec::new(),
                next_cursor: None,
            },
        };
        let encoded = serde_json::to_string(&response).expect("detail should serialize");
        let decoded: CenterDetailResponse =
            serde_json::from_str(&encoded).expect("detail should deserialize");
        assert_eq!(decoded.center.member_count, 108);
        assert!(decoded.events.items.is_empty());
    }

    #[test]
    fn session_contract_never_serializes_credentials() {
        let response = SessionResponse {
            authenticated: true,
            user: Some(SessionUser {
                id: "user-1".to_owned(),
                email: "member@example.test".to_owned(),
                display_name: "Member".to_owned(),
                verification_level: 45,
                role: "member".to_owned(),
            }),
        };
        let value = serde_json::to_value(response).expect("session should serialize");
        assert_eq!(value["authenticated"], true);
        assert!(value.get("token").is_none());
        assert!(value.get("csrfToken").is_none());
        assert!(value.get("nativeAccessToken").is_none());
    }
}
