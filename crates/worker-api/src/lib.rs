//! Cloudflare Worker API for the isolated Janata Rust proof of concept.
//!
//! The runtime adapter is intentionally thin. Exact routing, query validation,
//! deterministic seed construction, pagination, CORS, and cache choices remain
//! ordinary Rust so contributors can understand and test them on a host machine.

mod query;
mod seed;

use janata_api_contract::{ApiError, DiscoverResponse, FeedResponse, NotificationsResponse, Page};
use query::{QueryShape, ReadQuery, parse_read_query};
use serde_json::{Value, json};
use worker::js_sys::{Function, Reflect};
use worker::wasm_bindgen::{JsCast, JsValue};
use worker::{Context, Env, Headers, Request, Response, Result, console_error, console_log, event};

/// Stable service name returned by the health endpoint.
pub const SERVICE_NAME: &str = "janata-rust-poc";

/// Browser origins allowed to read the POC API.
///
/// These are exact origins, not suffix or wildcard matches. The production POC
/// is same-origin, while the loopback entries support Dioxus and Wrangler local
/// development on their conventional ports.
pub const ALLOWED_ORIGINS: &[&str] = &[
    "https://cmrust.sahasta.com",
    "http://localhost:8080",
    "http://127.0.0.1:8080",
    "http://localhost:8787",
    "http://127.0.0.1:8787",
];

const HEALTH_PATH: &str = "/api/health";
const DISCOVER_PATH: &str = "/api/v1/discover";
const EVENTS_PATH: &str = "/api/v1/events";
const FEED_PATH: &str = "/api/v1/community/feed";
const NOTIFICATIONS_PATH: &str = "/api/v1/notifications";
const BOOTSTRAP_PATH: &str = "/api/v1/bootstrap";
const ALLOWED_REQUEST_HEADERS: &[&str] = &["content-type", "x-request-id"];

/// Result of matching an HTTP method and path against the POC API surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiRoute {
    /// `GET /api/health`.
    Health,
    /// `GET /api/v1/discover`.
    Discover,
    /// `GET /api/v1/events`.
    Events,
    /// `GET /api/v1/community/feed`.
    Feed,
    /// `GET /api/v1/notifications`.
    Notifications,
    /// `GET /api/v1/bootstrap`.
    Bootstrap,
    /// CORS preflight for an existing read route.
    Preflight,
    /// A known versioned path requested with a method this read-only POC rejects.
    MethodNotAllowed,
    /// Any method/path pair not explicitly exposed by this POC.
    NotFound,
}

/// Response caching intent, kept explicit at each route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CachePolicy {
    /// Short browser/CDN caching for deterministic public discovery and feed data.
    PublicShortLived,
    /// Private data and errors must not be retained by shared or browser caches.
    PrivateNoStore,
}

impl CachePolicy {
    /// Returns the exact `Cache-Control` value applied to a response.
    #[must_use]
    pub const fn header_value(self) -> &'static str {
        match self {
            Self::PublicShortLived => "public, max-age=30, s-maxage=60, stale-while-revalidate=120",
            Self::PrivateNoStore => "private, no-store",
        }
    }
}

/// Matches the deliberately small API surface using exact method/path checks.
///
/// Versioned endpoints are read-only. A client attempting any other method on
/// one of those exact paths receives [`ApiRoute::MethodNotAllowed`]. Unknown
/// paths remain indistinguishable as [`ApiRoute::NotFound`].
#[must_use]
pub fn classify_route(method: &str, path: &str) -> ApiRoute {
    if method == "GET" {
        return match path {
            HEALTH_PATH => ApiRoute::Health,
            DISCOVER_PATH => ApiRoute::Discover,
            EVENTS_PATH => ApiRoute::Events,
            FEED_PATH => ApiRoute::Feed,
            NOTIFICATIONS_PATH => ApiRoute::Notifications,
            BOOTSTRAP_PATH => ApiRoute::Bootstrap,
            _ => ApiRoute::NotFound,
        };
    }

    if method == "OPTIONS" && is_known_path(path) {
        return ApiRoute::Preflight;
    }

    if is_versioned_path(path) {
        ApiRoute::MethodNotAllowed
    } else {
        ApiRoute::NotFound
    }
}

/// Returns an origin only when it is an exact member of [`ALLOWED_ORIGINS`].
#[must_use]
pub fn allowed_origin(origin: Option<&str>) -> Option<&str> {
    origin.filter(|candidate| ALLOWED_ORIGINS.contains(candidate))
}

/// Checks the browser's requested CORS headers against a small allowlist.
///
/// Header names are case-insensitive. Empty comma-separated entries are
/// ignored, but every actual name must be explicitly allowed.
#[must_use]
pub fn requested_headers_allowed(requested: Option<&str>) -> bool {
    requested.is_none_or(|headers| {
        headers
            .split(',')
            .map(str::trim)
            .filter(|header| !header.is_empty())
            .all(|header| {
                ALLOWED_REQUEST_HEADERS
                    .iter()
                    .any(|allowed| header.eq_ignore_ascii_case(allowed))
            })
    })
}

/// Evaluates the complete CORS preflight policy without runtime dependencies.
#[must_use]
pub fn cors_preflight_allowed(
    origin: Option<&str>,
    requested_method: Option<&str>,
    requested_headers: Option<&str>,
) -> bool {
    allowed_origin(origin).is_some()
        && requested_method == Some("GET")
        && requested_headers_allowed(requested_headers)
}

/// Validates the bounded character set used by Cloudflare Ray IDs.
#[must_use]
pub fn valid_ray_id(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.len() <= 64
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// Cloudflare's WebAssembly fetch entry point.
#[event(fetch)]
pub async fn fetch(request: Request, _env: Env, _context: Context) -> Result<Response> {
    let method = request.method().to_string();
    let path = request.path();
    let request_id = request_id(&request).unwrap_or_else(|error| {
        console_error!(
            "{}",
            json!({
                "event": "request_id_generation_failed",
                "error": error.to_string(),
            })
        );
        "request-id-unavailable".to_owned()
    });

    match dispatch(&request, &method, &path, &request_id) {
        Ok(response) => {
            console_log!(
                "{}",
                json!({
                    "event": "request_complete",
                    "request_id": request_id,
                    "method": method,
                    "path": path,
                    "status": response.status_code(),
                })
            );
            Ok(response)
        }
        Err(error) => {
            console_error!(
                "{}",
                json!({
                    "event": "request_failed",
                    "request_id": request_id,
                    "method": method,
                    "path": path,
                    "error": error.to_string(),
                })
            );
            error_response(
                "internal_error",
                "The request could not be completed.",
                500,
                &request_id,
                None,
            )
        }
    }
}

fn dispatch(request: &Request, method: &str, path: &str, request_id: &str) -> Result<Response> {
    let origin = request.headers().get("origin")?;
    let url = request.url()?;

    match classify_route(method, path) {
        ApiRoute::Health => health_response(request_id, origin.as_deref()),
        ApiRoute::Discover => match parse_read_query(&url, QueryShape::Discover) {
            Ok(query) => read_response(
                serde_json::to_value(discover_payload(query)?)?,
                CachePolicy::PublicShortLived,
                request_id,
                origin.as_deref(),
            ),
            Err(error) => invalid_query_response(&error, request_id, origin.as_deref()),
        },
        ApiRoute::Events => match parse_read_query(&url, QueryShape::Events) {
            Ok(query) => read_response(
                events_payload(query)?,
                CachePolicy::PublicShortLived,
                request_id,
                origin.as_deref(),
            ),
            Err(error) => invalid_query_response(&error, request_id, origin.as_deref()),
        },
        ApiRoute::Feed => match parse_read_query(&url, QueryShape::Page) {
            Ok(query) => read_response(
                serde_json::to_value(feed_payload(query)?)?,
                CachePolicy::PublicShortLived,
                request_id,
                origin.as_deref(),
            ),
            Err(error) => invalid_query_response(&error, request_id, origin.as_deref()),
        },
        ApiRoute::Notifications => match parse_read_query(&url, QueryShape::Page) {
            Ok(query) => read_response(
                serde_json::to_value(notifications_payload(query)?)?,
                CachePolicy::PrivateNoStore,
                request_id,
                origin.as_deref(),
            ),
            Err(error) => invalid_query_response(&error, request_id, origin.as_deref()),
        },
        ApiRoute::Bootstrap => match parse_read_query(&url, QueryShape::None) {
            Ok(_) => read_response(
                bootstrap_payload()?,
                CachePolicy::PrivateNoStore,
                request_id,
                origin.as_deref(),
            ),
            Err(error) => invalid_query_response(&error, request_id, origin.as_deref()),
        },
        ApiRoute::Preflight => preflight_response(request, request_id, origin.as_deref()),
        ApiRoute::MethodNotAllowed => {
            let mut response = error_response(
                "method_not_allowed",
                "This proof-of-concept endpoint is read-only; use GET.",
                405,
                request_id,
                origin.as_deref(),
            )?;
            response.headers_mut().set("Allow", "GET, OPTIONS")?;
            Ok(response)
        }
        ApiRoute::NotFound => error_response(
            "not_found",
            "No API route matches this request.",
            404,
            request_id,
            origin.as_deref(),
        ),
    }
}

fn health_response(request_id: &str, origin: Option<&str>) -> Result<Response> {
    read_response(
        json!({
            "service": SERVICE_NAME,
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "runtime": "cloudflare-workers-rs",
            "request_id": request_id,
        }),
        CachePolicy::PrivateNoStore,
        request_id,
        origin,
    )
}

fn preflight_response(
    request: &Request,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let requested_method = request.headers().get("access-control-request-method")?;
    let requested_headers = request.headers().get("access-control-request-headers")?;

    if !cors_preflight_allowed(
        origin,
        requested_method.as_deref(),
        requested_headers.as_deref(),
    ) {
        return error_response(
            "cors_forbidden",
            "The CORS preflight request is not allowed.",
            403,
            request_id,
            None,
        );
    }

    let mut response = Response::empty()?.with_status(204);
    apply_common_headers(
        response.headers_mut(),
        request_id,
        origin,
        CachePolicy::PrivateNoStore,
    )?;
    response
        .headers_mut()
        .set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
    response
        .headers_mut()
        .set("Access-Control-Allow-Headers", "Content-Type, X-Request-ID")?;
    response
        .headers_mut()
        .set("Access-Control-Max-Age", "600")?;
    Ok(response)
}

fn read_response(
    body: Value,
    cache_policy: CachePolicy,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let mut response = Response::from_json(&body)?;
    apply_common_headers(response.headers_mut(), request_id, origin, cache_policy)?;
    Ok(response)
}

fn error_response(
    code: &str,
    message: &str,
    status: u16,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let body = json!({
        "error": ApiError {
            code: code.to_owned(),
            message: message.to_owned(),
            request_id: request_id.to_owned(),
        }
    });
    let mut response = Response::from_json(&body)?.with_status(status);
    apply_common_headers(
        response.headers_mut(),
        request_id,
        origin,
        CachePolicy::PrivateNoStore,
    )?;
    Ok(response)
}

fn invalid_query_response(
    error: &query::QueryError,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    error_response("invalid_query", error.message(), 400, request_id, origin)
}

fn apply_common_headers(
    headers: &mut Headers,
    request_id: &str,
    origin: Option<&str>,
    cache_policy: CachePolicy,
) -> Result<()> {
    headers.set("Cache-Control", cache_policy.header_value())?;
    headers.set(
        "Content-Security-Policy",
        "default-src 'none'; base-uri 'none'; frame-ancestors 'none'",
    )?;
    headers.set("Cross-Origin-Resource-Policy", "same-origin")?;
    headers.set(
        "Permissions-Policy",
        "accelerometer=(), camera=(), geolocation=(), gyroscope=(), microphone=(), payment=(), usb=()",
    )?;
    headers.set("Referrer-Policy", "no-referrer")?;
    headers.set(
        "Strict-Transport-Security",
        "max-age=63072000; includeSubDomains; preload",
    )?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("X-Frame-Options", "DENY")?;
    headers.set("X-Request-ID", request_id)?;
    headers.set("Vary", "Origin")?;

    if let Some(origin) = allowed_origin(origin) {
        headers.set("Access-Control-Allow-Origin", origin)?;
        headers.set("Access-Control-Expose-Headers", "X-Request-ID")?;
    }

    Ok(())
}

fn discover_payload(query: ReadQuery) -> serde_json::Result<DiscoverResponse> {
    let mut discover = seed::discover()?;
    discover.events.retain(|event| {
        query.category.as_deref().is_none_or(|expected| {
            event
                .category
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
        })
    });
    discover.events.truncate(query.limit);
    Ok(discover)
}

fn events_payload(query: ReadQuery) -> serde_json::Result<Value> {
    let mut discover = seed::discover()?;
    discover.events.retain(|event| {
        query.category.as_deref().is_none_or(|expected| {
            event
                .category
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
        })
    });
    serde_json::to_value(paginate(&discover.events, query.offset, query.limit))
}

fn feed_payload(query: ReadQuery) -> serde_json::Result<FeedResponse> {
    let mut response = seed::feed()?;
    response.posts = paginate(&response.posts.items, query.offset, query.limit);
    Ok(response)
}

fn notifications_payload(query: ReadQuery) -> serde_json::Result<NotificationsResponse> {
    let mut response = seed::notifications()?;
    response.notifications = paginate(&response.notifications.items, query.offset, query.limit);
    Ok(response)
}

fn bootstrap_payload() -> serde_json::Result<Value> {
    Ok(json!({
        "discover": discover_payload(ReadQuery::default())?,
        "feed": feed_payload(ReadQuery::default())?,
        "notifications": notifications_payload(ReadQuery::default())?,
    }))
}

fn paginate<T: Clone>(items: &[T], offset: usize, limit: usize) -> Page<T> {
    let start = offset.min(items.len());
    let end = start.saturating_add(limit).min(items.len());
    Page {
        items: items[start..end].to_vec(),
        next_cursor: (end < items.len()).then(|| format!("offset:{end}")),
    }
}

fn is_known_path(path: &str) -> bool {
    path == HEALTH_PATH || is_versioned_path(path)
}

fn is_versioned_path(path: &str) -> bool {
    path == DISCOVER_PATH
        || path == EVENTS_PATH
        || path == FEED_PATH
        || path == NOTIFICATIONS_PATH
        || path == BOOTSTRAP_PATH
}

fn request_id(request: &Request) -> Result<String> {
    if let Some(ray_id) = request.headers().get("cf-ray")?
        && valid_ray_id(&ray_id)
    {
        return Ok(format!("cf-{ray_id}"));
    }

    random_uuid()
}

fn random_uuid() -> Result<String> {
    let global = worker::js_sys::global();
    let crypto = Reflect::get(&global, &JsValue::from_str("crypto"))?;
    let function =
        Reflect::get(&crypto, &JsValue::from_str("randomUUID"))?.dyn_into::<Function>()?;
    function
        .call0(&crypto)?
        .as_string()
        .ok_or_else(|| worker::Error::RustError("crypto.randomUUID returned a non-string".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_matching_is_exact_for_every_read_endpoint() {
        let reads = [
            (HEALTH_PATH, ApiRoute::Health),
            (DISCOVER_PATH, ApiRoute::Discover),
            (EVENTS_PATH, ApiRoute::Events),
            (FEED_PATH, ApiRoute::Feed),
            (NOTIFICATIONS_PATH, ApiRoute::Notifications),
            (BOOTSTRAP_PATH, ApiRoute::Bootstrap),
        ];
        for (path, route) in reads {
            assert_eq!(classify_route("GET", path), route);
            assert_eq!(classify_route("OPTIONS", path), ApiRoute::Preflight);
            assert_eq!(
                classify_route("GET", &format!("{path}/")),
                ApiRoute::NotFound
            );
        }
    }

    #[test]
    fn versioned_mutations_fail_explicitly_without_exposing_unknown_paths() {
        for method in ["POST", "PUT", "PATCH", "DELETE", "HEAD"] {
            assert_eq!(
                classify_route(method, EVENTS_PATH),
                ApiRoute::MethodNotAllowed
            );
        }
        assert_eq!(classify_route("POST", HEALTH_PATH), ApiRoute::NotFound);
        assert_eq!(classify_route("POST", "/api/v1/admin"), ApiRoute::NotFound);
        assert_eq!(
            classify_route("OPTIONS", "/api/v1/missing"),
            ApiRoute::NotFound
        );
    }

    #[test]
    fn cors_origins_are_exact() {
        for origin in ALLOWED_ORIGINS {
            assert_eq!(allowed_origin(Some(origin)), Some(*origin));
        }

        assert_eq!(allowed_origin(None), None);
        assert_eq!(
            allowed_origin(Some("https://cmrust.sahasta.com.evil.test")),
            None
        );
        assert_eq!(allowed_origin(Some("https://CMRUST.SAHASTA.COM")), None);
        assert_eq!(allowed_origin(Some("http://localhost:3000")), None);
    }

    #[test]
    fn preflight_policy_requires_exact_method_origin_and_headers() {
        assert!(cors_preflight_allowed(
            Some("http://localhost:8080"),
            Some("GET"),
            Some("Content-Type, X-Request-ID")
        ));
        assert!(!cors_preflight_allowed(
            Some("http://localhost:8080"),
            Some("POST"),
            None
        ));
        assert!(!cors_preflight_allowed(
            Some("https://evil.test"),
            Some("GET"),
            None
        ));
        assert!(!cors_preflight_allowed(
            Some("http://localhost:8080"),
            Some("GET"),
            Some("Authorization")
        ));
    }

    #[test]
    fn preflight_headers_are_allowlisted_case_insensitively() {
        assert!(requested_headers_allowed(None));
        assert!(requested_headers_allowed(Some("")));
        assert!(requested_headers_allowed(Some(
            "Content-Type, X-Request-ID"
        )));
        assert!(requested_headers_allowed(Some("content-type,x-request-id")));
        assert!(!requested_headers_allowed(Some("Authorization")));
        assert!(!requested_headers_allowed(Some("Content-Type, X-Unsafe")));
    }

    #[test]
    fn cache_policies_do_not_mix_public_and_notification_data() {
        assert!(
            CachePolicy::PublicShortLived
                .header_value()
                .starts_with("public")
        );
        assert_eq!(
            CachePolicy::PrivateNoStore.header_value(),
            "private, no-store"
        );
    }

    #[test]
    fn pagination_is_bounded_and_uses_an_opaque_next_cursor() {
        let page = paginate(&[1, 2, 3], 0, 2);
        assert_eq!(page.items, vec![1, 2]);
        assert_eq!(page.next_cursor.as_deref(), Some("offset:2"));

        let final_page = paginate(&[1, 2, 3], 2, 50);
        assert_eq!(final_page.items, vec![3]);
        assert_eq!(final_page.next_cursor, None);

        let beyond_end = paginate(&[1, 2, 3], 10_000, 1);
        assert!(beyond_end.items.is_empty());
        assert_eq!(beyond_end.next_cursor, None);
    }

    #[test]
    fn event_filter_and_typed_contracts_are_stable() {
        let discover = discover_payload(ReadQuery {
            limit: 1,
            offset: 0,
            category: Some("satsang".to_owned()),
        })
        .expect("discover payload should build");
        assert_eq!(discover.centers.len(), 2);
        assert_eq!(discover.events.len(), 1);
        assert_eq!(discover.events[0].category.as_deref(), Some("satsang"));

        let events = events_payload(ReadQuery {
            limit: 2,
            offset: 0,
            category: None,
        })
        .expect("event page should build");
        assert_eq!(events["items"].as_array().map(Vec::len), Some(2));
        assert_eq!(events["nextCursor"], "offset:2");
    }

    #[test]
    fn notifications_report_total_unread_count_across_pages() {
        let response = notifications_payload(ReadQuery {
            limit: 1,
            offset: 1,
            category: None,
        })
        .expect("notification payload should build");
        assert_eq!(response.notifications.items.len(), 1);
        assert_eq!(response.unread_count, 2);
    }

    #[test]
    fn bootstrap_is_one_typed_aggregate() {
        let value = bootstrap_payload().expect("bootstrap contract should serialize");
        assert!(value.get("discover").is_some());
        assert!(value.get("feed").is_some());
        assert!(value.get("notifications").is_some());
        assert_eq!(value.as_object().map(serde_json::Map::len), Some(3));
    }

    #[test]
    fn structured_error_contains_one_support_request_id() {
        let envelope = json!({
            "error": ApiError {
                code: "invalid_query".to_owned(),
                message: "Fix the query.".to_owned(),
                request_id: "request-123".to_owned(),
            }
        });
        let value = serde_json::to_value(envelope).expect("error should serialize");
        assert_eq!(value["error"]["code"], "invalid_query");
        assert_eq!(value["error"]["requestId"], "request-123");
    }

    #[test]
    fn ray_ids_are_bounded_and_header_safe() {
        assert!(valid_ray_id("abc123-SJC"));
        assert!(!valid_ray_id(""));
        assert!(!valid_ray_id("abc_123"));
        assert!(!valid_ray_id("abc\nInjected"));
        assert!(!valid_ray_id(&"a".repeat(65)));
    }
}
