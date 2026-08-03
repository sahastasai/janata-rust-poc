//! Stable Serde contracts shared by the Dioxus UI and Cloudflare Worker.
//!
//! Contracts deliberately use domain types instead of loose JSON maps. This
//! makes breaking changes visible to the compiler and documentation generator.

use janata_domain::{BoardPost, Center, Event, Notification, UserProfile};
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

/// Paginated response shared by list endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    /// Current page of records.
    pub items: Vec<T>,
    /// Opaque cursor for the next page.
    pub next_cursor: Option<String>,
}

/// Credentials submitted to the authentication endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    /// Email address or username.
    pub login: String,
    /// User-entered password; never logged or persisted in this representation.
    pub password: String,
}

/// Session summary returned after authentication or token refresh.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    /// Authenticated member profile.
    pub user: UserProfile,
    /// CSRF token for cookie-authenticated web mutations.
    pub csrf_token: Option<String>,
    /// Short-lived bearer token used only by native clients.
    pub native_access_token: Option<String>,
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

/// POC endpoints implemented during the compile-first foundation phase.
pub const FOUNDATION_ENDPOINTS: &[EndpointSpec] = &[EndpointSpec {
    method: "GET",
    path: "/api/health",
    authorization: "public",
    summary: "Report POC service and revision status",
}];

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
}
