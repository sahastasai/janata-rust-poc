//! Cloudflare Worker API for the isolated Janata Rust proof of concept.
//!
//! The runtime adapter is intentionally thin. Exact routing, canonical ID and
//! query validation, deterministic seed construction, pagination, CORS, and
//! cache choices remain ordinary Rust so contributors can test them on a host.

mod auth;
mod query;
mod seed;

use janata_api_contract::{
    ApiError, ApiErrorEnvelope, CenterDetailResponse, DiscoverResponse, EventDetailResponse,
    FeedResponse, NotificationsResponse, Page,
};
use janata_domain::{Center, CenterId, Event, EventId};
use query::{QueryError, QueryShape, ReadQuery, parse_read_query};
use serde_json::{Value, json};
use worker::js_sys::{Function, Reflect};
use worker::wasm_bindgen::{JsCast, JsValue};
use worker::{Context, Env, Headers, Request, Response, Result, console_error, event};

/// Stable service name returned by the health endpoint.
pub const SERVICE_NAME: &str = "janata-rust-poc";

/// Browser origins allowed to read the POC API.
///
/// These are exact origins, not suffix or wildcard matches. The production POC
/// is same-origin, while the loopback entries support Dioxus and Wrangler local
/// development on their conventional ports.
pub const ALLOWED_ORIGINS: &[&str] = &[
    "https://cmrust.sahasta.com",
    "https://localhost:8787",
    "https://127.0.0.1:8787",
    "http://localhost:8080",
    "http://127.0.0.1:8080",
    "http://localhost:8787",
    "http://127.0.0.1:8787",
];

const HEALTH_PATH: &str = "/api/health";
const DISCOVER_PATH: &str = "/api/v1/discover";
const CENTERS_PATH: &str = "/api/v1/centers";
const CENTER_DETAIL_PREFIX: &str = "/api/v1/centers/";
const EVENTS_PATH: &str = "/api/v1/events";
const EVENT_DETAIL_PREFIX: &str = "/api/v1/events/";
const FEED_PATH: &str = "/api/v1/community/feed";
const NOTIFICATIONS_PATH: &str = "/api/v1/notifications";
const BOOTSTRAP_PATH: &str = "/api/v1/bootstrap";
const COMPAT_CENTERS_PATH: &str = "/api/centers";
const COMPAT_EVENTS_PATH: &str = "/api/fetchAllEvents";
const COMPAT_CENTER_PATH: &str = "/api/fetchCenter";
const COMPAT_EVENT_PATH: &str = "/api/fetchEvent";
const COMPAT_CENTER_EVENTS_PATH: &str = "/api/fetchEventsByCenter";
const AUTH_VALIDATE_INVITE_PATH: &str = "/api/v1/auth/invite/validate";
const AUTH_REGISTER_PATH: &str = "/api/v1/auth/register";
const AUTH_LOGIN_PATH: &str = "/api/v1/auth/login";
const AUTH_SESSION_PATH: &str = "/api/v1/auth/session";
const AUTH_LOGOUT_PATH: &str = "/api/v1/auth/logout";
const ALLOWED_REQUEST_HEADERS: &[&str] = &["content-type", "x-csrf-token", "x-request-id"];

/// Result of matching an HTTP method and path against the POC API surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiRoute {
    /// `GET /api/health`.
    Health,
    /// `GET /api/v1/discover`.
    Discover,
    /// `GET /api/v1/centers`.
    Centers,
    /// `GET /api/v1/centers/:id`.
    CenterDetail,
    /// `GET /api/v1/events`.
    Events,
    /// `GET /api/v1/events/:id`.
    EventDetail,
    /// `GET /api/v1/community/feed`.
    Feed,
    /// `GET /api/v1/notifications`.
    Notifications,
    /// `GET /api/v1/bootstrap`.
    Bootstrap,
    /// Compatibility `GET /api/centers`.
    CompatibilityCenters,
    /// Compatibility `GET /api/fetchAllEvents`.
    CompatibilityEvents,
    /// Compatibility `GET /api/fetchCenter?centerID=`.
    CompatibilityCenter,
    /// Compatibility `GET /api/fetchEvent?id=`.
    CompatibilityEvent,
    /// Compatibility `GET /api/fetchEventsByCenter?centerID=`.
    CompatibilityCenterEvents,
    /// `POST /api/v1/auth/invite/validate`.
    AuthValidateInvite,
    /// `POST /api/v1/auth/register`.
    AuthRegister,
    /// `POST /api/v1/auth/login`.
    AuthLogin,
    /// `GET /api/v1/auth/session`.
    AuthSession,
    /// `POST /api/v1/auth/logout`.
    AuthLogout,
    /// CORS preflight for an existing route.
    Preflight,
    /// A known path requested with a method this POC rejects.
    MethodNotAllowed,
    /// Any method/path pair not explicitly exposed by this POC.
    NotFound,
}

/// Response caching intent, kept explicit at each route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CachePolicy {
    /// Short browser/CDN caching for deterministic public discovery data.
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

/// Pure, client-safe problem description used by the runtime and host tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HttpProblem {
    code: &'static str,
    message: &'static str,
    status: u16,
}

impl HttpProblem {
    const fn invalid_center_id() -> Self {
        Self {
            code: "invalid_id",
            message: "Center ID must be a lowercase hyphenated UUID.",
            status: 400,
        }
    }

    const fn invalid_event_id() -> Self {
        Self {
            code: "invalid_id",
            message: "Event ID must be a lowercase hyphenated UUID.",
            status: 400,
        }
    }

    const fn center_not_found() -> Self {
        Self {
            code: "not_found",
            message: "Center not found.",
            status: 404,
        }
    }

    const fn event_not_found() -> Self {
        Self {
            code: "not_found",
            message: "Event not found.",
            status: 404,
        }
    }
}

/// Matches the deliberately small API surface using exact method/path checks.
///
/// Every exposed route is read-only. Dynamic paths accept exactly one non-empty
/// segment; canonical UUID validation happens before lookup in the handler.
#[must_use]
pub fn classify_route(method: &str, path: &str) -> ApiRoute {
    if let Some((route, expected_method)) = classify_auth_path(path) {
        return match method {
            actual if actual == expected_method => route,
            "OPTIONS" => ApiRoute::Preflight,
            _ => ApiRoute::MethodNotAllowed,
        };
    }
    let read_route = classify_read_path(path);
    match method {
        "GET" => read_route,
        "OPTIONS" if read_route != ApiRoute::NotFound => ApiRoute::Preflight,
        _ if read_route != ApiRoute::NotFound => ApiRoute::MethodNotAllowed,
        _ => ApiRoute::NotFound,
    }
}

fn classify_auth_path(path: &str) -> Option<(ApiRoute, &'static str)> {
    match path {
        AUTH_VALIDATE_INVITE_PATH => Some((ApiRoute::AuthValidateInvite, "POST")),
        AUTH_REGISTER_PATH => Some((ApiRoute::AuthRegister, "POST")),
        AUTH_LOGIN_PATH => Some((ApiRoute::AuthLogin, "POST")),
        AUTH_SESSION_PATH => Some((ApiRoute::AuthSession, "GET")),
        AUTH_LOGOUT_PATH => Some((ApiRoute::AuthLogout, "POST")),
        _ => None,
    }
}

fn expected_method(path: &str) -> Option<&'static str> {
    classify_auth_path(path)
        .map(|(_, method)| method)
        .or_else(|| (classify_read_path(path) != ApiRoute::NotFound).then_some("GET"))
}

fn classify_read_path(path: &str) -> ApiRoute {
    match path {
        HEALTH_PATH => ApiRoute::Health,
        DISCOVER_PATH => ApiRoute::Discover,
        CENTERS_PATH => ApiRoute::Centers,
        EVENTS_PATH => ApiRoute::Events,
        FEED_PATH => ApiRoute::Feed,
        NOTIFICATIONS_PATH => ApiRoute::Notifications,
        BOOTSTRAP_PATH => ApiRoute::Bootstrap,
        COMPAT_CENTERS_PATH => ApiRoute::CompatibilityCenters,
        COMPAT_EVENTS_PATH => ApiRoute::CompatibilityEvents,
        COMPAT_CENTER_PATH => ApiRoute::CompatibilityCenter,
        COMPAT_EVENT_PATH => ApiRoute::CompatibilityEvent,
        COMPAT_CENTER_EVENTS_PATH => ApiRoute::CompatibilityCenterEvents,
        _ if one_dynamic_segment(path, CENTER_DETAIL_PREFIX).is_some() => ApiRoute::CenterDetail,
        _ if one_dynamic_segment(path, EVENT_DETAIL_PREFIX).is_some() => ApiRoute::EventDetail,
        _ => ApiRoute::NotFound,
    }
}

fn one_dynamic_segment<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    path.strip_prefix(prefix)
        .filter(|segment| !segment.is_empty() && !segment.contains('/'))
}

fn parse_center_path(path: &str) -> std::result::Result<CenterId, HttpProblem> {
    one_dynamic_segment(path, CENTER_DETAIL_PREFIX)
        .and_then(|segment| CenterId::parse_canonical(segment).ok())
        .ok_or_else(HttpProblem::invalid_center_id)
}

fn parse_event_path(path: &str) -> std::result::Result<EventId, HttpProblem> {
    one_dynamic_segment(path, EVENT_DETAIL_PREFIX)
        .and_then(|segment| EventId::parse_canonical(segment).ok())
        .ok_or_else(HttpProblem::invalid_event_id)
}

/// Returns an origin only when it is an exact member of [`ALLOWED_ORIGINS`].
#[must_use]
pub fn allowed_origin(origin: Option<&str>) -> Option<&str> {
    origin.filter(|candidate| ALLOWED_ORIGINS.contains(candidate))
}

/// Checks the browser's requested CORS headers against a small allowlist.
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
    let mut request = request;
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

    match dispatch(&mut request, &_env, &method, &path, &request_id).await {
        Ok(response) => Ok(response),
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

async fn dispatch(
    request: &mut Request,
    env: &Env,
    method: &str,
    path: &str,
    request_id: &str,
) -> Result<Response> {
    let origin = request.headers().get("origin")?;
    let origin = origin.as_deref();
    let route = classify_route(method, path);

    match route {
        ApiRoute::Health => health_response(request_id, origin),
        ApiRoute::Discover => {
            with_query(request, QueryShape::Discover, request_id, origin, |query| {
                public_response(
                    serde_json::to_value(discover_payload(query)?)?,
                    route,
                    request_id,
                    origin,
                )
            })
        }
        ApiRoute::Centers => with_query(request, QueryShape::Page, request_id, origin, |query| {
            public_response(
                serde_json::to_value(centers_payload(query)?)?,
                route,
                request_id,
                origin,
            )
        }),
        ApiRoute::CenterDetail => {
            let center_id = match parse_center_path(path) {
                Ok(id) => id,
                Err(problem) => return problem_response(problem, request_id, origin),
            };
            with_query(request, QueryShape::Page, request_id, origin, |query| {
                match center_detail_payload(center_id, query)? {
                    Some(detail) => {
                        public_response(serde_json::to_value(detail)?, route, request_id, origin)
                    }
                    None => problem_response(HttpProblem::center_not_found(), request_id, origin),
                }
            })
        }
        ApiRoute::Events => with_query(request, QueryShape::Events, request_id, origin, |query| {
            public_response(
                serde_json::to_value(events_payload(query)?)?,
                route,
                request_id,
                origin,
            )
        }),
        ApiRoute::EventDetail => {
            let event_id = match parse_event_path(path) {
                Ok(id) => id,
                Err(problem) => return problem_response(problem, request_id, origin),
            };
            with_query(request, QueryShape::None, request_id, origin, |_| {
                match event_detail_payload(event_id)? {
                    Some(detail) => {
                        public_response(serde_json::to_value(detail)?, route, request_id, origin)
                    }
                    None => problem_response(HttpProblem::event_not_found(), request_id, origin),
                }
            })
        }
        ApiRoute::Feed => with_query(request, QueryShape::Page, request_id, origin, |query| {
            public_response(
                serde_json::to_value(feed_payload(query)?)?,
                route,
                request_id,
                origin,
            )
        }),
        ApiRoute::Notifications => {
            with_query(request, QueryShape::Page, request_id, origin, |query| {
                read_response(
                    serde_json::to_value(notifications_payload(query)?)?,
                    CachePolicy::PrivateNoStore,
                    request_id,
                    origin,
                )
            })
        }
        ApiRoute::Bootstrap => with_query(request, QueryShape::None, request_id, origin, |_| {
            read_response(
                bootstrap_payload()?,
                CachePolicy::PrivateNoStore,
                request_id,
                origin,
            )
        }),
        ApiRoute::CompatibilityCenters => with_query(
            request,
            QueryShape::CompatibilityPage,
            request_id,
            origin,
            |query| {
                public_response(
                    compatibility_centers_payload(query)?,
                    route,
                    request_id,
                    origin,
                )
            },
        ),
        ApiRoute::CompatibilityEvents => with_query(
            request,
            QueryShape::CompatibilityEvents,
            request_id,
            origin,
            |query| {
                public_response(
                    compatibility_events_payload(query)?,
                    route,
                    request_id,
                    origin,
                )
            },
        ),
        ApiRoute::CompatibilityCenter => with_query(
            request,
            QueryShape::CompatibilityCenter,
            request_id,
            origin,
            |query| match compatibility_center_payload(
                query.center_id.expect("validated compatibility center ID"),
            )? {
                Some(value) => public_response(value, route, request_id, origin),
                None => problem_response(HttpProblem::center_not_found(), request_id, origin),
            },
        ),
        ApiRoute::CompatibilityEvent => with_query(
            request,
            QueryShape::CompatibilityEvent,
            request_id,
            origin,
            |query| match compatibility_event_payload(
                query.event_id.expect("validated compatibility event ID"),
            )? {
                Some(value) => public_response(value, route, request_id, origin),
                None => problem_response(HttpProblem::event_not_found(), request_id, origin),
            },
        ),
        ApiRoute::CompatibilityCenterEvents => with_query(
            request,
            QueryShape::CompatibilityEventsByCenter,
            request_id,
            origin,
            |query| {
                public_response(
                    compatibility_center_events_payload(query)?,
                    route,
                    request_id,
                    origin,
                )
            },
        ),
        ApiRoute::AuthValidateInvite => {
            auth_dispatch(
                auth::AuthAction::ValidateInvite,
                request,
                env,
                request_id,
                origin,
            )
            .await
        }
        ApiRoute::AuthRegister => {
            auth_dispatch(auth::AuthAction::Register, request, env, request_id, origin).await
        }
        ApiRoute::AuthLogin => {
            auth_dispatch(auth::AuthAction::Login, request, env, request_id, origin).await
        }
        ApiRoute::AuthSession => {
            auth_dispatch(auth::AuthAction::Session, request, env, request_id, origin).await
        }
        ApiRoute::AuthLogout => {
            auth_dispatch(auth::AuthAction::Logout, request, env, request_id, origin).await
        }
        ApiRoute::Preflight => preflight_response(request, path, request_id, origin),
        ApiRoute::MethodNotAllowed => {
            let allowed = expected_method(path).unwrap_or("GET");
            let mut response = error_response(
                "method_not_allowed",
                "Use the documented HTTP method for this endpoint.",
                405,
                request_id,
                origin,
            )?;
            response
                .headers_mut()
                .set("Allow", &format!("{allowed}, OPTIONS"))?;
            Ok(response)
        }
        ApiRoute::NotFound => error_response(
            "not_found",
            "No API route matches this request.",
            404,
            request_id,
            origin,
        ),
    }
}

async fn auth_dispatch(
    action: auth::AuthAction,
    request: &mut Request,
    env: &Env,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    if action != auth::AuthAction::Session && allowed_origin(origin).is_none() {
        return credentialed_error_response(
            "origin_forbidden",
            "This authentication request must come from an allowed browser origin.",
            403,
            request_id,
            None,
        );
    }

    match auth::handle(action, request, env).await? {
        Ok(reply) => auth_success_response(reply, request_id, origin),
        Err(problem) => credentialed_error_response(
            problem.code,
            problem.message,
            problem.status,
            request_id,
            origin,
        ),
    }
}

fn with_query<F>(
    request: &Request,
    shape: QueryShape,
    request_id: &str,
    origin: Option<&str>,
    handler: F,
) -> Result<Response>
where
    F: FnOnce(ReadQuery) -> Result<Response>,
{
    match request_query(request, shape)? {
        Ok(query) => handler(query),
        Err(error) => invalid_query_response(&error, request_id, origin),
    }
}

fn request_query(
    request: &Request,
    shape: QueryShape,
) -> Result<std::result::Result<ReadQuery, QueryError>> {
    Ok(parse_read_query(&request.url()?, shape))
}

fn success_cache_policy(route: ApiRoute) -> CachePolicy {
    match route {
        ApiRoute::Discover
        | ApiRoute::Centers
        | ApiRoute::CenterDetail
        | ApiRoute::Events
        | ApiRoute::EventDetail
        | ApiRoute::Feed
        | ApiRoute::CompatibilityCenters
        | ApiRoute::CompatibilityEvents
        | ApiRoute::CompatibilityCenter
        | ApiRoute::CompatibilityEvent
        | ApiRoute::CompatibilityCenterEvents => CachePolicy::PublicShortLived,
        _ => CachePolicy::PrivateNoStore,
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
    path: &str,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let requested_method = request.headers().get("access-control-request-method")?;
    let requested_headers = request.headers().get("access-control-request-headers")?;

    if allowed_origin(origin).is_none()
        || requested_method.as_deref() != expected_method(path)
        || !requested_headers_allowed(requested_headers.as_deref())
    {
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
    response.headers_mut().set(
        "Access-Control-Allow-Methods",
        &format!("{}, OPTIONS", expected_method(path).unwrap_or("GET")),
    )?;
    response.headers_mut().set(
        "Access-Control-Allow-Headers",
        "Content-Type, X-CSRF-Token, X-Request-ID",
    )?;
    if classify_auth_path(path).is_some() {
        response
            .headers_mut()
            .set("Access-Control-Allow-Credentials", "true")?;
    }
    response
        .headers_mut()
        .set("Access-Control-Max-Age", "600")?;
    Ok(response)
}

fn auth_success_response(
    reply: auth::AuthReply,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let mut response = Response::from_json(&reply.body)?.with_status(reply.status);
    apply_credentialed_headers(response.headers_mut(), request_id, origin)?;
    for cookie in &reply.set_cookies {
        response.headers_mut().append("Set-Cookie", cookie)?;
    }
    Ok(response)
}

fn credentialed_error_response(
    code: &str,
    message: &str,
    status: u16,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let mut response = error_response(code, message, status, request_id, origin)?;
    apply_credentialed_headers(response.headers_mut(), request_id, origin)?;
    Ok(response)
}

fn apply_credentialed_headers(
    headers: &mut Headers,
    request_id: &str,
    origin: Option<&str>,
) -> Result<()> {
    apply_common_headers(headers, request_id, origin, CachePolicy::PrivateNoStore)?;
    headers.set("Vary", "Origin, Cookie")?;
    if allowed_origin(origin).is_some() {
        headers.set("Access-Control-Allow-Credentials", "true")?;
    }
    Ok(())
}

fn public_response(
    body: Value,
    route: ApiRoute,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    read_response(body, success_cache_policy(route), request_id, origin)
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
    let body = ApiErrorEnvelope {
        error: ApiError {
            code: code.to_owned(),
            message: message.to_owned(),
            request_id: request_id.to_owned(),
        },
    };
    let mut response = Response::from_json(&body)?.with_status(status);
    apply_common_headers(
        response.headers_mut(),
        request_id,
        origin,
        CachePolicy::PrivateNoStore,
    )?;
    Ok(response)
}

fn problem_response(
    problem: HttpProblem,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    error_response(
        problem.code,
        problem.message,
        problem.status,
        request_id,
        origin,
    )
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
    filter_events(&mut discover.events, &query);
    discover.centers.truncate(query.limit);
    discover.events.truncate(query.limit);
    Ok(discover)
}

fn centers_payload(query: ReadQuery) -> serde_json::Result<Page<Center>> {
    let discover = seed::discover()?;
    Ok(paginate(&discover.centers, query.offset, query.limit))
}

fn center_detail_payload(
    center_id: CenterId,
    query: ReadQuery,
) -> serde_json::Result<Option<CenterDetailResponse>> {
    let discover = seed::discover()?;
    let Some(center) = discover
        .centers
        .into_iter()
        .find(|center| center.id == center_id)
    else {
        return Ok(None);
    };
    let events = discover
        .events
        .into_iter()
        .filter(|event| event.center_id == Some(center_id))
        .collect::<Vec<_>>();
    Ok(Some(CenterDetailResponse {
        center,
        events: paginate(&events, query.offset, query.limit),
    }))
}

fn events_payload(query: ReadQuery) -> serde_json::Result<Page<Event>> {
    let mut events = seed::discover()?.events;
    filter_events(&mut events, &query);
    Ok(paginate(&events, query.offset, query.limit))
}

fn event_detail_payload(event_id: EventId) -> serde_json::Result<Option<EventDetailResponse>> {
    Ok(seed::discover()?
        .events
        .into_iter()
        .find(|event| event.id == event_id)
        .map(|event| EventDetailResponse { event }))
}

fn filter_events(events: &mut Vec<Event>, query: &ReadQuery) {
    events.retain(|event| {
        query.category.as_deref().is_none_or(|expected| {
            event
                .category
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
        }) && query
            .center_id
            .is_none_or(|expected| event.center_id == Some(expected))
    });
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

fn compatibility_centers_payload(query: ReadQuery) -> serde_json::Result<Value> {
    let centers = seed::discover()?.centers;
    let total = centers.len();
    let page = paginate(&centers, query.offset, query.limit);
    Ok(json!({
        "centers": page.items.iter().map(compatibility_center).collect::<Vec<_>>(),
        "total": total,
        "limit": query.limit,
        "offset": query.offset,
    }))
}

fn compatibility_events_payload(query: ReadQuery) -> serde_json::Result<Value> {
    let events = seed::discover()?.events;
    let total = events.len();
    let page = paginate(&events, query.offset, query.limit);
    Ok(json!({
        "message": "Success",
        "events": page.items.iter().map(compatibility_event).collect::<Vec<_>>(),
        "total": total,
        "limit": query.limit,
        "offset": query.offset,
    }))
}

fn compatibility_center_payload(center_id: CenterId) -> serde_json::Result<Option<Value>> {
    Ok(seed::discover()?
        .centers
        .iter()
        .find(|center| center.id == center_id)
        .map(|center| json!({ "message": "Success", "center": compatibility_center(center) })))
}

fn compatibility_event_payload(event_id: EventId) -> serde_json::Result<Option<Value>> {
    Ok(seed::discover()?
        .events
        .iter()
        .find(|event| event.id == event_id)
        .map(|event| json!({ "message": "Success", "event": compatibility_event(event) })))
}

fn compatibility_center_events_payload(query: ReadQuery) -> serde_json::Result<Value> {
    let center_id = query.center_id.expect("validated compatibility center ID");
    let events = seed::discover()?
        .events
        .into_iter()
        .filter(|event| event.center_id == Some(center_id))
        .collect::<Vec<_>>();
    let total = events.len();
    let page = paginate(&events, query.offset, query.limit);
    Ok(json!({
        "message": "Success",
        "events": page.items.iter().map(compatibility_event).collect::<Vec<_>>(),
        "total": total,
        "limit": query.limit,
        "offset": query.offset,
    }))
}

fn compatibility_center(center: &Center) -> Value {
    json!({
        "centerID": center.id,
        "name": center.name,
        "latitude": center.latitude,
        "longitude": center.longitude,
        "address": center.address,
        "website": center.website,
        "phone": center.phone,
        "image": center.image_url,
        "acharya": center.acharya,
        "pointOfContact": center.point_of_contact,
        "description": center.description,
        "memberCount": center.member_count,
        "isVerified": center.verified,
    })
}

fn compatibility_event(event: &Event) -> Value {
    json!({
        "eventID": event.id,
        "title": event.title,
        "description": event.description,
        "date": event.date,
        "endDate": event.end_date,
        "isRecurring": event.recurring,
        "timeLabel": event.time_label,
        "latitude": event.latitude,
        "longitude": event.longitude,
        "address": event.address,
        "centerID": event.center_id,
        "tier": 0,
        "peopleAttending": event.attendee_count,
        "pointOfContact": event.point_of_contact,
        "image": event.image_url,
        "category": event.category,
        "createdBy": event.created_by,
        "externalUrl": event.external_url,
        "signupUrl": event.signup_url,
        "allowJanataSignup": event.allow_janata_signup,
        "isOfficial": event.official,
        "requiresVerified": event.requires_verified,
    })
}

fn paginate<T: Clone>(items: &[T], offset: usize, limit: usize) -> Page<T> {
    let start = offset.min(items.len());
    let end = start.saturating_add(limit).min(items.len());
    Page {
        items: items[start..end].to_vec(),
        next_cursor: (end < items.len()).then(|| format!("offset:{end}")),
    }
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

    const CENTER_ID: &str = "00000000-0000-0000-0000-000000000065";
    const EVENT_ID: &str = "00000000-0000-0000-0000-0000000000c9";

    #[test]
    fn route_matching_is_exact_for_every_read_endpoint() {
        let reads = [
            (HEALTH_PATH, ApiRoute::Health),
            (DISCOVER_PATH, ApiRoute::Discover),
            (CENTERS_PATH, ApiRoute::Centers),
            (
                &format!("{CENTER_DETAIL_PREFIX}{CENTER_ID}"),
                ApiRoute::CenterDetail,
            ),
            (EVENTS_PATH, ApiRoute::Events),
            (
                &format!("{EVENT_DETAIL_PREFIX}{EVENT_ID}"),
                ApiRoute::EventDetail,
            ),
            (FEED_PATH, ApiRoute::Feed),
            (NOTIFICATIONS_PATH, ApiRoute::Notifications),
            (BOOTSTRAP_PATH, ApiRoute::Bootstrap),
            (COMPAT_CENTERS_PATH, ApiRoute::CompatibilityCenters),
            (COMPAT_EVENTS_PATH, ApiRoute::CompatibilityEvents),
            (COMPAT_CENTER_PATH, ApiRoute::CompatibilityCenter),
            (COMPAT_EVENT_PATH, ApiRoute::CompatibilityEvent),
            (
                COMPAT_CENTER_EVENTS_PATH,
                ApiRoute::CompatibilityCenterEvents,
            ),
        ];
        for (path, route) in reads {
            assert_eq!(classify_route("GET", path), route, "GET {path}");
            assert_eq!(
                classify_route("OPTIONS", path),
                ApiRoute::Preflight,
                "OPTIONS {path}"
            );
            for method in ["POST", "PUT", "PATCH", "DELETE", "HEAD"] {
                assert_eq!(
                    classify_route(method, path),
                    ApiRoute::MethodNotAllowed,
                    "{method} {path}"
                );
            }
        }
    }

    #[test]
    fn authentication_routes_accept_only_their_documented_methods() {
        let posts = [
            (AUTH_VALIDATE_INVITE_PATH, ApiRoute::AuthValidateInvite),
            (AUTH_REGISTER_PATH, ApiRoute::AuthRegister),
            (AUTH_LOGIN_PATH, ApiRoute::AuthLogin),
            (AUTH_LOGOUT_PATH, ApiRoute::AuthLogout),
        ];
        for (path, route) in posts {
            assert_eq!(classify_route("POST", path), route);
            assert_eq!(classify_route("OPTIONS", path), ApiRoute::Preflight);
            assert_eq!(classify_route("GET", path), ApiRoute::MethodNotAllowed);
            assert_eq!(expected_method(path), Some("POST"));
        }

        assert_eq!(
            classify_route("GET", AUTH_SESSION_PATH),
            ApiRoute::AuthSession
        );
        assert_eq!(
            classify_route("POST", AUTH_SESSION_PATH),
            ApiRoute::MethodNotAllowed
        );
        assert_eq!(expected_method(AUTH_SESSION_PATH), Some("GET"));
        assert!(requested_headers_allowed(Some(
            "Content-Type, X-CSRF-Token, X-Request-ID"
        )));
    }

    #[test]
    fn dynamic_routes_reject_extra_or_missing_segments() {
        for path in [
            CENTER_DETAIL_PREFIX,
            "/api/v1/centers/",
            &format!("{CENTER_DETAIL_PREFIX}{CENTER_ID}/"),
            &format!("{CENTER_DETAIL_PREFIX}{CENTER_ID}/events"),
            EVENT_DETAIL_PREFIX,
            &format!("{EVENT_DETAIL_PREFIX}{EVENT_ID}/"),
        ] {
            assert_eq!(classify_route("GET", path), ApiRoute::NotFound, "{path}");
            assert_eq!(
                classify_route("OPTIONS", path),
                ApiRoute::NotFound,
                "{path}"
            );
        }
    }

    #[test]
    fn dynamic_ids_are_canonical_before_lookup() {
        assert_eq!(
            parse_center_path(&format!("{CENTER_DETAIL_PREFIX}{CENTER_ID}"))
                .map(|id| id.to_string()),
            Ok(CENTER_ID.to_owned())
        );
        assert_eq!(
            parse_event_path(&format!("{EVENT_DETAIL_PREFIX}{EVENT_ID}")).map(|id| id.to_string()),
            Ok(EVENT_ID.to_owned())
        );
        assert_eq!(
            parse_center_path("/api/v1/centers/00000000000000000000000000000065"),
            Err(HttpProblem::invalid_center_id())
        );
        assert_eq!(
            parse_event_path("/api/v1/events/NOT-A-UUID"),
            Err(HttpProblem::invalid_event_id())
        );
    }

    #[test]
    fn cors_origins_and_preflights_are_exact() {
        for origin in ALLOWED_ORIGINS {
            assert_eq!(allowed_origin(Some(origin)), Some(*origin));
        }
        assert_eq!(allowed_origin(None), None);
        assert_eq!(
            allowed_origin(Some("https://cmrust.sahasta.com.evil.test")),
            None
        );
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
        assert!(!requested_headers_allowed(Some("Authorization")));
    }

    #[test]
    fn public_discovery_successes_are_cacheable_but_private_routes_and_errors_are_not() {
        let public = [
            ApiRoute::Discover,
            ApiRoute::Centers,
            ApiRoute::CenterDetail,
            ApiRoute::Events,
            ApiRoute::EventDetail,
            ApiRoute::CompatibilityCenters,
            ApiRoute::CompatibilityEvents,
            ApiRoute::CompatibilityCenter,
            ApiRoute::CompatibilityEvent,
            ApiRoute::CompatibilityCenterEvents,
        ];
        for route in public {
            assert_eq!(success_cache_policy(route), CachePolicy::PublicShortLived);
        }
        assert_eq!(
            success_cache_policy(ApiRoute::Health),
            CachePolicy::PrivateNoStore
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
        let beyond_end = paginate(&[1, 2, 3], 10_000, 1);
        assert!(beyond_end.items.is_empty());
        assert_eq!(beyond_end.next_cursor, None);
    }

    #[test]
    fn typed_center_and_event_filters_are_stable() {
        let center_id = CenterId::parse_canonical(CENTER_ID).expect("fixture center ID");
        let discover = discover_payload(ReadQuery {
            limit: 1,
            category: Some("meditation".to_owned()),
            center_id: Some(center_id),
            ..ReadQuery::default()
        })
        .expect("discover payload should build");
        assert_eq!(discover.centers.len(), 1);
        assert_eq!(discover.events.len(), 1);
        assert_eq!(discover.events[0].center_id, Some(center_id));

        let events = events_payload(ReadQuery {
            limit: 2,
            center_id: Some(center_id),
            ..ReadQuery::default()
        })
        .expect("event page should build");
        assert_eq!(events.items.len(), 2);
        assert_eq!(events.next_cursor, None);
    }

    #[test]
    fn detail_lookup_distinguishes_missing_resources() {
        let center_id = CenterId::parse_canonical(CENTER_ID).expect("fixture center ID");
        let detail = center_detail_payload(center_id, ReadQuery::default())
            .expect("center detail should build")
            .expect("center should exist");
        assert_eq!(detail.center.id, center_id);
        assert_eq!(detail.events.items.len(), 2);

        let missing = EventId::parse_canonical("00000000-0000-0000-0000-000000009999")
            .expect("canonical missing ID");
        assert!(
            event_detail_payload(missing)
                .expect("event lookup should build")
                .is_none()
        );
        assert_eq!(HttpProblem::event_not_found().status, 404);
        assert_eq!(HttpProblem::event_not_found().code, "not_found");
    }

    #[test]
    fn compatibility_aliases_keep_reference_envelopes_and_remain_bounded() {
        let centers = compatibility_centers_payload(ReadQuery {
            limit: 1,
            ..ReadQuery::default()
        })
        .expect("center alias should build");
        assert_eq!(centers["centers"].as_array().map(Vec::len), Some(1));
        assert_eq!(centers["centers"][0]["centerID"], CENTER_ID);
        assert_eq!(centers["limit"], 1);
        assert_eq!(centers["total"], 2);

        let event_id = EventId::parse_canonical(EVENT_ID).expect("fixture event ID");
        let event = compatibility_event_payload(event_id)
            .expect("event alias should build")
            .expect("event should exist");
        assert_eq!(event["message"], "Success");
        assert_eq!(event["event"]["eventID"], EVENT_ID);
        assert_eq!(event["event"]["peopleAttending"], 18);
    }

    #[test]
    fn structured_errors_contain_one_support_request_id() {
        let envelope = ApiErrorEnvelope {
            error: ApiError {
                code: "invalid_query".to_owned(),
                message: "Fix the query.".to_owned(),
                request_id: "request-123".to_owned(),
            },
        };
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
