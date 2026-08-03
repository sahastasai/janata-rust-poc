//! Small cross-platform client for the public Rust Worker discovery API.
//!
//! Web requests derive their absolute URL from the active browser origin, so
//! they cannot be redirected to another service by runtime configuration.
//! Native packages have no trustworthy implicit origin, so they require an
//! explicit compile-time `JANATA_API_BASE_URL`; without one the UI shows an
//! unavailable state and never substitutes sample data.

use std::fmt::{Display, Formatter};

use janata_api_contract::{
    ApiErrorEnvelope, CenterDetailResponse, DiscoverResponse, EventDetailResponse,
};
use janata_domain::{CenterId, EventId};
use serde::de::DeserializeOwned;

const DISCOVER_PATH: &str = "/api/v1/discover";
const CENTERS_PATH: &str = "/api/v1/centers";
const EVENTS_PATH: &str = "/api/v1/events";

/// Failure that can be explained directly in a public, read-only screen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ApiClientError {
    /// A mobile build does not provide a safe API origin.
    Unavailable(String),
    /// A direct-link identifier is not canonical.
    InvalidId(String),
    /// The Worker returned a structured 404.
    NotFound(String),
    /// The network request could not complete.
    Request(String),
    /// The Worker returned a non-success response or malformed contract.
    Response(String),
}

impl Display for ApiClientError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message)
            | Self::InvalidId(message)
            | Self::NotFound(message)
            | Self::Request(message)
            | Self::Response(message) => message.fmt(formatter),
        }
    }
}

/// Loads the bounded center/event snapshot used by Discover.
pub(crate) async fn fetch_discover(
    category: Option<String>,
    center_id: Option<CenterId>,
) -> Result<DiscoverResponse, ApiClientError> {
    let mut query = vec![("limit", "50".to_owned())];
    if let Some(category) = category.filter(|category| !category.is_empty()) {
        query.push(("category", category));
    }
    if let Some(center_id) = center_id {
        query.push(("centerId", center_id.to_string()));
    }
    get_json(DISCOVER_PATH, &query).await
}

/// Loads one center and its bounded upcoming events.
pub(crate) async fn fetch_center_detail(
    raw_id: &str,
) -> Result<CenterDetailResponse, ApiClientError> {
    let center_id = CenterId::parse_canonical(raw_id).map_err(|_| {
        ApiClientError::InvalidId(
            "This center link is invalid. Return to Discover and choose a center.".to_owned(),
        )
    })?;
    let path = format!("{CENTERS_PATH}/{center_id}");
    get_json(&path, &[("limit", "50".to_owned())]).await
}

/// Loads one public event without registration or attendee-roster state.
pub(crate) async fn fetch_event_detail(
    raw_id: &str,
) -> Result<EventDetailResponse, ApiClientError> {
    let event_id = EventId::parse_canonical(raw_id).map_err(|_| {
        ApiClientError::InvalidId(
            "This event link is invalid. Return to Discover and choose an event.".to_owned(),
        )
    })?;
    let path = format!("{EVENTS_PATH}/{event_id}");
    get_json(&path, &[]).await
}

async fn get_json<T>(path: &str, query: &[(&str, String)]) -> Result<T, ApiClientError>
where
    T: DeserializeOwned,
{
    let base = configured_api_base()?;
    let url = join_api_path(&base, path)?;
    let response = reqwest::Client::new()
        .get(url)
        .query(query)
        .send()
        .await
        .map_err(|error| {
            ApiClientError::Request(format!(
                "The public Janata service could not be reached: {error}"
            ))
        })?;
    let status = response.status();

    if !status.is_success() {
        let fallback = format!("The public Janata service returned HTTP {status}.");
        let message = response
            .json::<ApiErrorEnvelope>()
            .await
            .map(|envelope| envelope.error.message)
            .unwrap_or(fallback);
        return if status == reqwest::StatusCode::NOT_FOUND {
            Err(ApiClientError::NotFound(message))
        } else {
            Err(ApiClientError::Response(message))
        };
    }

    response.json::<T>().await.map_err(|error| {
        ApiClientError::Response(format!(
            "The public Janata service returned an unreadable response: {error}"
        ))
    })
}

fn configured_api_base() -> Result<String, ApiClientError> {
    #[cfg(feature = "mobile")]
    {
        resolve_api_base(true, option_env!("JANATA_API_BASE_URL"))
    }

    #[cfg(not(feature = "mobile"))]
    {
        #[cfg(target_arch = "wasm32")]
        {
            let origin = web_sys::window()
                .ok_or_else(|| {
                    ApiClientError::Unavailable(
                        "Public discovery requires an active browser window.".to_owned(),
                    )
                })?
                .location()
                .origin()
                .map_err(|_| {
                    ApiClientError::Unavailable(
                        "The browser origin could not be read for public discovery.".to_owned(),
                    )
                })?;
            resolve_web_origin(&origin)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            resolve_api_base(false, None)
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn resolve_web_origin(origin: &str) -> Result<String, ApiClientError> {
    let origin = origin.trim_end_matches('/');
    if !(origin.starts_with("https://")
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:"))
        || origin.contains(['?', '#'])
    {
        return Err(ApiClientError::Unavailable(
            "The browser did not provide a safe same-origin API address.".to_owned(),
        ));
    }
    Ok(origin.to_owned())
}

#[cfg(any(feature = "mobile", not(target_arch = "wasm32")))]
fn resolve_api_base(native: bool, configured: Option<&str>) -> Result<String, ApiClientError> {
    if !native {
        return Ok(String::new());
    }

    let base = configured
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .map(|candidate| candidate.trim_end_matches('/'))
        .ok_or_else(|| {
            ApiClientError::Unavailable(
                "Public discovery is unavailable in this mobile build. Rebuild with an explicit JANATA_API_BASE_URL.".to_owned(),
            )
        })?;

    let secure = base.starts_with("https://");
    let local_development =
        base.starts_with("http://localhost:") || base.starts_with("http://127.0.0.1:");
    if (!secure && !local_development) || base.contains(['?', '#']) {
        return Err(ApiClientError::Unavailable(
            "The configured mobile API address must be HTTPS (or an explicit loopback development address).".to_owned(),
        ));
    }

    Ok(base.to_owned())
}

fn join_api_path(base: &str, path: &str) -> Result<String, ApiClientError> {
    if !path.starts_with('/') || path.contains(['?', '#']) {
        return Err(ApiClientError::Response(
            "The application attempted to use an invalid API path.".to_owned(),
        ));
    }
    Ok(format!("{base}{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_uses_same_origin_paths() {
        let base = resolve_api_base(false, Some("https://ignored.example"))
            .expect("web base should resolve");
        assert!(base.is_empty());
        assert_eq!(
            join_api_path(&base, DISCOVER_PATH),
            Ok(DISCOVER_PATH.to_owned())
        );
        assert_eq!(
            resolve_web_origin("https://cmrust.sahasta.com/"),
            Ok("https://cmrust.sahasta.com".to_owned())
        );
        assert!(resolve_web_origin("http://example.test").is_err());
    }

    #[test]
    fn native_fails_closed_without_an_explicit_safe_origin() {
        assert!(matches!(
            resolve_api_base(true, None),
            Err(ApiClientError::Unavailable(_))
        ));
        assert!(matches!(
            resolve_api_base(true, Some("http://example.test")),
            Err(ApiClientError::Unavailable(_))
        ));
        assert_eq!(
            resolve_api_base(true, Some("https://cmrust.sahasta.com/")),
            Ok("https://cmrust.sahasta.com".to_owned())
        );
        assert_eq!(
            resolve_api_base(true, Some("http://127.0.0.1:8787")),
            Ok("http://127.0.0.1:8787".to_owned())
        );
    }

    #[test]
    fn direct_link_ids_are_validated_before_request_construction() {
        assert!(CenterId::parse_canonical("not-a-center").is_err());
        assert!(EventId::parse_canonical("000000000000000000000000000000c9").is_err());
        assert!(join_api_path("", "api/v1/events").is_err());
    }
}
