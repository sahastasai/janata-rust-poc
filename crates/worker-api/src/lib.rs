//! Cloudflare Worker API for the isolated Janata Rust proof of concept.
//!
//! This crate intentionally keeps routing and CORS policy independent of the
//! Workers runtime. Those decisions can therefore be unit-tested on a normal
//! Rust host, while the small fetch adapter is compiled to WebAssembly.

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
const ALLOWED_REQUEST_HEADERS: &[&str] = &["content-type", "x-request-id"];

/// Result of matching an HTTP method and path against the POC API surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiRoute {
    /// `GET /api/health`.
    Health,
    /// CORS preflight for `/api/health`.
    HealthPreflight,
    /// Any method/path pair not explicitly exposed by this POC.
    NotFound,
}

/// Matches the deliberately small API surface using exact method/path checks.
///
/// Unsupported methods return [`ApiRoute::NotFound`] rather than advertising
/// which methods exist. This is the contract requested for the POC.
pub fn classify_route(method: &str, path: &str) -> ApiRoute {
    match (method, path) {
        ("GET", HEALTH_PATH) => ApiRoute::Health,
        ("OPTIONS", HEALTH_PATH) => ApiRoute::HealthPreflight,
        _ => ApiRoute::NotFound,
    }
}

/// Returns an origin only when it is an exact member of [`ALLOWED_ORIGINS`].
pub fn allowed_origin(origin: Option<&str>) -> Option<&str> {
    origin.filter(|candidate| ALLOWED_ORIGINS.contains(candidate))
}

/// Checks the browser's requested CORS headers against a small allowlist.
///
/// Header names are case-insensitive. Empty comma-separated entries are
/// ignored, but every actual name must be explicitly allowed.
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

/// Validates the bounded character set used by Cloudflare Ray IDs.
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
            json_response(
                json!({
                    "error": {
                        "code": "internal_error",
                        "message": "The request could not be completed.",
                    },
                    "request_id": request_id,
                }),
                500,
                &request_id,
                None,
            )
        }
    }
}

fn dispatch(request: &Request, method: &str, path: &str, request_id: &str) -> Result<Response> {
    let origin = request.headers().get("origin")?;

    match classify_route(method, path) {
        ApiRoute::Health => json_response(
            json!({
                "service": SERVICE_NAME,
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION"),
                "runtime": "cloudflare-workers-rs",
                "request_id": request_id,
            }),
            200,
            request_id,
            origin.as_deref(),
        ),
        ApiRoute::HealthPreflight => preflight_response(request, request_id, origin.as_deref()),
        ApiRoute::NotFound => json_response(
            json!({
                "error": {
                    "code": "not_found",
                    "message": "No API route matches this request.",
                },
                "request_id": request_id,
            }),
            404,
            request_id,
            origin.as_deref(),
        ),
    }
}

fn preflight_response(
    request: &Request,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let requested_method = request.headers().get("access-control-request-method")?;
    let requested_headers = request.headers().get("access-control-request-headers")?;
    let allowed = allowed_origin(origin).is_some()
        && requested_method.as_deref() == Some("GET")
        && requested_headers_allowed(requested_headers.as_deref());

    if !allowed {
        return json_response(
            json!({
                "error": {
                    "code": "cors_forbidden",
                    "message": "The CORS preflight request is not allowed.",
                },
                "request_id": request_id,
            }),
            403,
            request_id,
            None,
        );
    }

    let mut response = Response::empty()?.with_status(204);
    apply_common_headers(response.headers_mut(), request_id, origin)?;
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

fn json_response(
    body: Value,
    status: u16,
    request_id: &str,
    origin: Option<&str>,
) -> Result<Response> {
    let mut response = Response::from_json(&body)?.with_status(status);
    apply_common_headers(response.headers_mut(), request_id, origin)?;
    Ok(response)
}

fn apply_common_headers(
    headers: &mut Headers,
    request_id: &str,
    origin: Option<&str>,
) -> Result<()> {
    headers.set("Cache-Control", "no-store")?;
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
    fn route_matching_is_exact() {
        assert_eq!(classify_route("GET", "/api/health"), ApiRoute::Health);
        assert_eq!(
            classify_route("OPTIONS", "/api/health"),
            ApiRoute::HealthPreflight
        );
        assert_eq!(classify_route("POST", "/api/health"), ApiRoute::NotFound);
        assert_eq!(classify_route("GET", "/api/health/"), ApiRoute::NotFound);
        assert_eq!(classify_route("GET", "/api/missing"), ApiRoute::NotFound);
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
    fn ray_ids_are_bounded_and_header_safe() {
        assert!(valid_ray_id("abc123-SJC"));
        assert!(!valid_ray_id(""));
        assert!(!valid_ray_id("abc_123"));
        assert!(!valid_ray_id("abc\nInjected"));
        assert!(!valid_ray_id(&"a".repeat(65)));
    }
}
