//! Local-only browser authentication backed by the isolated `AUTH_DB` D1 binding.
//!
//! The browser receives an opaque session cookie and a separate double-submit
//! CSRF cookie. Only SHA-256 digests are stored in D1. Passwords are derived
//! with PBKDF2-SHA-256 using Web Crypto so the Worker never ships a private
//! cryptographic implementation.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use futures_util::StreamExt as _;
use janata_api_contract::{
    AuthenticateRequest, InviteValidationResponse, LogoutResponse, RegisterRequest,
    RegisterResponse, SessionResponse, SessionUser, ValidateInviteRequest,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use worker::js_sys::{Array, ArrayBuffer, Function, Object, Reflect, Uint8Array};
use worker::wasm_bindgen::{JsCast, JsValue};
use worker::{D1Database, Env, Request, Result};

const AUTH_DB_BINDING: &str = "AUTH_DB";
const AUTH_KDF_LIMITER_BINDING: &str = "AUTH_KDF_LIMITER";
const AUTH_ACCOUNT_LIMITER_BINDING: &str = "AUTH_ACCOUNT_LIMITER";
const MAX_JSON_BYTES: usize = 16 * 1024;
const PASSWORD_ITERATIONS: u32 = 600_000;
const PASSWORD_SALT_BYTES: usize = 16;
const PASSWORD_HASH_BYTES: usize = 32;
const SESSION_TOKEN_BYTES: usize = 32;
const SESSION_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;
const LOGIN_FAILURE_LIMIT: i32 = 5;
const LOGIN_COOLDOWN_SECONDS: i64 = 15 * 60;
const DUMMY_USER_ID: &str = "00000000-0000-0000-0000-000000000000";
const SESSION_COOKIE: &str = "__Host-janata_session";
const CSRF_COOKIE: &str = "__Host-janata_csrf";

// A fixed, valid derivation keeps unknown-account login timing close to known
// accounts without storing or exposing a dummy credential.
const DUMMY_SALT: [u8; PASSWORD_SALT_BYTES] = [
    0x4e, 0xc6, 0xe7, 0x80, 0x34, 0x25, 0x23, 0x44, 0xf6, 0x2c, 0xa1, 0x1d, 0x09, 0x3d, 0x91, 0x81,
];

/// Authentication route selected by the top-level Worker dispatcher.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthAction {
    ValidateInvite,
    Register,
    Login,
    Session,
    Logout,
}

/// Successful authentication result before HTTP security headers are applied.
#[derive(Debug)]
pub(crate) struct AuthReply {
    pub(crate) body: Value,
    pub(crate) status: u16,
    pub(crate) set_cookies: Vec<String>,
}

impl AuthReply {
    fn json<T: serde::Serialize>(body: T, status: u16) -> Result<Self> {
        Ok(Self {
            body: serde_json::to_value(body)?,
            status,
            set_cookies: Vec::new(),
        })
    }

    fn with_session_cookies(mut self, tokens: &SessionTokens) -> Self {
        self.set_cookies.push(format!(
            "{SESSION_COOKIE}={}; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age={SESSION_TTL_SECONDS}",
            tokens.session
        ));
        self.set_cookies.push(format!(
            "{CSRF_COOKIE}={}; Path=/; Secure; SameSite=Lax; Max-Age={SESSION_TTL_SECONDS}",
            tokens.csrf
        ));
        self
    }

    fn with_cleared_cookies(mut self) -> Self {
        self.set_cookies.push(format!(
            "{SESSION_COOKIE}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0"
        ));
        self.set_cookies.push(format!(
            "{CSRF_COOKIE}=; Path=/; Secure; SameSite=Lax; Max-Age=0"
        ));
        self
    }
}

/// Client-safe authentication failure. Internal D1 and crypto errors remain
/// ordinary Worker errors and are converted to the generic top-level 500.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuthProblem {
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
    pub(crate) status: u16,
}

impl AuthProblem {
    const fn new(code: &'static str, message: &'static str, status: u16) -> Self {
        Self {
            code,
            message,
            status,
        }
    }
}

type AuthResult = std::result::Result<AuthReply, AuthProblem>;

#[derive(Debug, Deserialize)]
struct InviteRow {
    inviter_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginUserRow {
    id: String,
    email_normalized: String,
    display_name: String,
    verification_level: i32,
    role: String,
    token_version: i32,
    password_salt: String,
    password_hash: String,
    password_iterations: i32,
    login_blocked_until: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SessionRow {
    session_id: String,
    user_id: String,
    email_normalized: String,
    display_name: String,
    verification_level: i32,
    role: String,
    last_seen_at: f64,
}

#[derive(Debug, Deserialize)]
struct LogoutSessionRow {
    session_id: String,
    csrf_hash: String,
}

#[derive(Debug)]
struct SessionTokens {
    session: String,
    csrf: String,
}

/// Executes one bounded authentication operation against the isolated D1
/// binding. Mutation origin checks happen in the dispatcher before this call.
pub(crate) async fn handle(
    action: AuthAction,
    request: &mut Request,
    env: &Env,
) -> Result<AuthResult> {
    let db = env.d1(AUTH_DB_BINDING)?;
    match action {
        AuthAction::ValidateInvite => validate_invite(request, &db).await,
        AuthAction::Register => register(request, &db, env).await,
        AuthAction::Login => login(request, &db, env).await,
        AuthAction::Session => session(request, &db).await,
        AuthAction::Logout => logout(request, &db).await,
    }
}

async fn validate_invite(request: &mut Request, db: &D1Database) -> Result<AuthResult> {
    let input: ValidateInviteRequest = match read_json(request).await? {
        Ok(input) => input,
        Err(problem) => return Ok(Err(problem)),
    };
    let code = match bounded_invite_code(&input.code) {
        Ok(code) => code,
        Err(problem) => return Ok(Err(problem)),
    };
    let code_hash = sha256_b64(code.as_bytes()).await?;
    let now = unix_seconds();
    let row = db
        .prepare(
            "SELECT inviter_name FROM invite_codes \
             WHERE code_hash = ?1 AND is_active = 1 AND revoked_at IS NULL \
             AND expires_at > ?2 AND (max_uses IS NULL OR use_count < max_uses)",
        )
        .bind(&[JsValue::from_str(&code_hash), js_number(now)])?
        .first::<InviteRow>(None)
        .await?;

    Ok(Ok(AuthReply::json(
        InviteValidationResponse {
            valid: row.is_some(),
            inviter_name: row.and_then(|invite| invite.inviter_name),
        },
        200,
    )?))
}

async fn register(request: &mut Request, db: &D1Database, env: &Env) -> Result<AuthResult> {
    let input: RegisterRequest = match read_json(request).await? {
        Ok(input) => input,
        Err(problem) => return Ok(Err(problem)),
    };
    let email = match normalized_email(&input.email) {
        Ok(email) => email,
        Err(problem) => return Ok(Err(problem)),
    };
    let display_name = match bounded_display_name(&input.display_name) {
        Ok(name) => name,
        Err(problem) => return Ok(Err(problem)),
    };
    if let Err(problem) = valid_new_password(&input.password) {
        return Ok(Err(problem));
    }
    if !account_rate_allowed(env, "register", &email).await? {
        return Ok(Err(rate_limited()));
    }
    let invite_code = match bounded_invite_code(&input.invite_code) {
        Ok(code) => code,
        Err(problem) => return Ok(Err(problem)),
    };
    if !kdf_rate_allowed(env).await? {
        return Ok(Err(rate_limited()));
    }

    let salt = random_bytes(PASSWORD_SALT_BYTES)?;
    let password_hash = pbkdf2(&input.password, &salt, PASSWORD_ITERATIONS).await?;
    let salt_b64 = URL_SAFE_NO_PAD.encode(salt);
    let hash_b64 = URL_SAFE_NO_PAD.encode(password_hash);
    let invite_hash = sha256_b64(invite_code.as_bytes()).await?;
    let user_id = super::random_uuid()?;
    let now = unix_seconds();

    // D1 batches are atomic. The INSERT can only materialize from a currently
    // valid invite; the paired UPDATE uses the same predicate and observes the
    // INSERT's serialized state inside the transaction.
    let insert = db
        .prepare(
            "INSERT INTO auth_users (id, email_normalized, display_name, \
             verification_level, role, token_version, password_salt, password_hash, \
             password_iterations, is_active, failed_login_count, login_blocked_until, \
             created_at, updated_at) \
             SELECT ?1, ?2, ?3, verification_level, role, 1, ?4, ?5, ?6, 1, 0, NULL, ?7, ?7 \
             FROM invite_codes WHERE code_hash = ?8 AND is_active = 1 \
             AND revoked_at IS NULL AND expires_at > ?7 \
             AND (max_uses IS NULL OR use_count < max_uses) \
             AND NOT EXISTS (SELECT 1 FROM auth_users WHERE email_normalized = ?2)",
        )
        .bind(&[
            JsValue::from_str(&user_id),
            JsValue::from_str(&email),
            JsValue::from_str(&display_name),
            JsValue::from_str(&salt_b64),
            JsValue::from_str(&hash_b64),
            JsValue::from_f64(f64::from(PASSWORD_ITERATIONS)),
            js_number(now),
            JsValue::from_str(&invite_hash),
        ])?;
    let consume = db
        .prepare(
            "UPDATE invite_codes SET use_count = use_count + 1 \
             WHERE code_hash = ?1 AND is_active = 1 AND revoked_at IS NULL \
             AND expires_at > ?2 AND (max_uses IS NULL OR use_count < max_uses) \
             AND EXISTS (SELECT 1 FROM auth_users WHERE id = ?3 \
             AND email_normalized = ?4 AND created_at = ?2)",
        )
        .bind(&[
            JsValue::from_str(&invite_hash),
            js_number(now),
            JsValue::from_str(&user_id),
            JsValue::from_str(&email),
        ])?;
    let results = db.batch(vec![insert, consume]).await?;
    let inserted = results
        .first()
        .and_then(|result| result.meta().ok().flatten())
        .and_then(|meta| meta.changes)
        .unwrap_or_default();
    if inserted != 1 {
        return Ok(Err(AuthProblem::new(
            "registration_unavailable",
            "Registration could not be completed with these details.",
            409,
        )));
    }

    Ok(Ok(AuthReply::json(
        RegisterResponse {
            registered: true,
            email,
        },
        201,
    )?))
}

async fn login(request: &mut Request, db: &D1Database, env: &Env) -> Result<AuthResult> {
    let input: AuthenticateRequest = match read_json(request).await? {
        Ok(input) => input,
        Err(problem) => return Ok(Err(problem)),
    };
    let email = normalized_email(&input.login).unwrap_or_default();
    let password_chars = input.password.chars().count();
    if password_chars == 0 || password_chars > 128 || input.password.len() > 512 {
        return Ok(Err(invalid_credentials()));
    }
    if !account_rate_allowed(env, "login", &email).await? {
        return Ok(Err(rate_limited()));
    }

    let user = db
        .prepare(
            "SELECT id, email_normalized, display_name, verification_level, role, \
             token_version, password_salt, password_hash, password_iterations, \
             login_blocked_until FROM auth_users \
             WHERE email_normalized = ?1 AND is_active = 1 LIMIT 1",
        )
        .bind(&[JsValue::from_str(&email)])?
        .first::<LoginUserRow>(None)
        .await?;

    let (salt, iterations, expected_hash) = user
        .as_ref()
        .and_then(|row| {
            let salt = URL_SAFE_NO_PAD.decode(&row.password_salt).ok()?;
            let hash = URL_SAFE_NO_PAD.decode(&row.password_hash).ok()?;
            let iterations = u32::try_from(row.password_iterations).ok()?;
            (salt.len() == PASSWORD_SALT_BYTES && hash.len() == PASSWORD_HASH_BYTES)
                .then_some((salt, iterations, hash))
        })
        .unwrap_or_else(|| {
            (
                DUMMY_SALT.to_vec(),
                PASSWORD_ITERATIONS,
                vec![0_u8; PASSWORD_HASH_BYTES],
            )
        });
    if !kdf_rate_allowed(env).await? {
        return Ok(Err(rate_limited()));
    }
    let candidate = pbkdf2(&input.password, &salt, iterations).await?;
    let password_matches = timing_safe_equal(&candidate, &expected_hash)?;
    let now = unix_seconds();

    let Some(user) = user else {
        record_failed_login(db, DUMMY_USER_ID, now).await?;
        return Ok(Err(invalid_credentials()));
    };
    if user
        .login_blocked_until
        .is_some_and(|blocked_until| blocked_until > now as f64)
    {
        // Keep the same response as every other failed login so a cooldown
        // cannot be used to confirm whether an account exists.
        record_failed_login(db, DUMMY_USER_ID, now).await?;
        return Ok(Err(invalid_credentials()));
    }
    if !password_matches {
        record_failed_login(db, &user.id, now).await?;
        return Ok(Err(invalid_credentials()));
    }

    let tokens = SessionTokens {
        session: URL_SAFE_NO_PAD.encode(random_bytes(SESSION_TOKEN_BYTES)?),
        csrf: URL_SAFE_NO_PAD.encode(random_bytes(SESSION_TOKEN_BYTES)?),
    };
    let session_hash = sha256_b64(tokens.session.as_bytes()).await?;
    let csrf_hash = sha256_b64(tokens.csrf.as_bytes()).await?;
    let session_id = super::random_uuid()?;
    let expires_at = now + SESSION_TTL_SECONDS;

    let reset_failures = db
        .prepare(
            "UPDATE auth_users SET failed_login_count = 0, login_blocked_until = NULL, \
             updated_at = ?1 WHERE id = ?2",
        )
        .bind(&[js_number(now), JsValue::from_str(&user.id)])?;
    let prune_expired = db
        .prepare(
            "DELETE FROM web_sessions WHERE user_id = ?1 \
             AND (revoked_at IS NOT NULL OR expires_at <= ?2)",
        )
        .bind(&[JsValue::from_str(&user.id), js_number(now)])?;
    let prune_oldest = db
        .prepare(
            "DELETE FROM web_sessions WHERE id IN (SELECT id FROM web_sessions \
             WHERE user_id = ?1 AND revoked_at IS NULL AND expires_at > ?2 \
             ORDER BY created_at DESC LIMIT -1 OFFSET 9)",
        )
        .bind(&[JsValue::from_str(&user.id), js_number(now)])?;
    let insert_session = db
        .prepare(
            "INSERT INTO web_sessions (id, user_id, token_hash, csrf_hash, token_version, \
             created_at, last_seen_at, expires_at, revoked_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, NULL)",
        )
        .bind(&[
            JsValue::from_str(&session_id),
            JsValue::from_str(&user.id),
            JsValue::from_str(&session_hash),
            JsValue::from_str(&csrf_hash),
            JsValue::from_f64(f64::from(user.token_version)),
            js_number(now),
            js_number(expires_at),
        ])?;
    db.batch(vec![
        reset_failures,
        prune_expired,
        prune_oldest,
        insert_session,
    ])
    .await?;

    let response = SessionResponse {
        authenticated: true,
        user: Some(session_user(&user)),
    };
    Ok(Ok(
        AuthReply::json(response, 200)?.with_session_cookies(&tokens)
    ))
}

async fn session(request: &Request, db: &D1Database) -> Result<AuthResult> {
    let Some(token) = cookie_value(request, SESSION_COOKIE)? else {
        return Ok(Ok(AuthReply::json(guest_session(), 200)?));
    };
    let token_hash = sha256_b64(token.as_bytes()).await?;
    let now = unix_seconds();
    let row = db
        .prepare(
            "SELECT s.id AS session_id, u.id AS user_id, u.email_normalized, \
             u.display_name, u.verification_level, u.role, s.last_seen_at \
             FROM web_sessions s JOIN auth_users u ON u.id = s.user_id \
             WHERE s.token_hash = ?1 AND s.revoked_at IS NULL AND s.expires_at > ?2 \
             AND u.is_active = 1 AND s.token_version = u.token_version LIMIT 1",
        )
        .bind(&[JsValue::from_str(&token_hash), js_number(now)])?
        .first::<SessionRow>(None)
        .await?;
    let Some(row) = row else {
        return Ok(Ok(
            AuthReply::json(guest_session(), 200)?.with_cleared_cookies()
        ));
    };

    // Throttle this write to keep a session check inexpensive under normal
    // navigation while still retaining useful local POC activity metadata.
    if row.last_seen_at + 900.0 < now as f64 {
        db.prepare("UPDATE web_sessions SET last_seen_at = ?1 WHERE id = ?2")
            .bind(&[js_number(now), JsValue::from_str(&row.session_id)])?
            .run()
            .await?;
    }

    Ok(Ok(AuthReply::json(
        SessionResponse {
            authenticated: true,
            user: Some(SessionUser {
                id: row.user_id,
                email: row.email_normalized,
                display_name: row.display_name,
                verification_level: row.verification_level,
                role: row.role,
            }),
        },
        200,
    )?))
}

async fn logout(request: &Request, db: &D1Database) -> Result<AuthResult> {
    let session_token = cookie_value(request, SESSION_COOKIE)?;
    let csrf_cookie = cookie_value(request, CSRF_COOKIE)?;
    let csrf_header = request.headers().get("x-csrf-token")?;

    let Some(session_token) = session_token else {
        return Ok(Ok(AuthReply::json(
            LogoutResponse { logged_out: true },
            200,
        )?
        .with_cleared_cookies()));
    };
    let (Some(csrf_cookie), Some(csrf_header)) = (csrf_cookie, csrf_header) else {
        return Ok(Err(csrf_problem()));
    };
    if !timing_safe_equal(csrf_cookie.as_bytes(), csrf_header.as_bytes())? {
        return Ok(Err(csrf_problem()));
    }

    let token_hash = sha256_b64(session_token.as_bytes()).await?;
    let row = db
        .prepare(
            "SELECT id AS session_id, csrf_hash FROM web_sessions \
             WHERE token_hash = ?1 AND revoked_at IS NULL LIMIT 1",
        )
        .bind(&[JsValue::from_str(&token_hash)])?
        .first::<LogoutSessionRow>(None)
        .await?;
    if let Some(row) = row {
        let presented_hash = sha256_b64(csrf_header.as_bytes()).await?;
        if !timing_safe_equal(presented_hash.as_bytes(), row.csrf_hash.as_bytes())? {
            return Ok(Err(csrf_problem()));
        }
        let now = unix_seconds();
        db.prepare("UPDATE web_sessions SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL")
            .bind(&[js_number(now), JsValue::from_str(&row.session_id)])?
            .run()
            .await?;
    }

    Ok(Ok(AuthReply::json(
        LogoutResponse { logged_out: true },
        200,
    )?
    .with_cleared_cookies()))
}

async fn record_failed_login(db: &D1Database, user_id: &str, now: i64) -> Result<()> {
    let blocked_until = now + LOGIN_COOLDOWN_SECONDS;
    db.prepare(
        "UPDATE auth_users SET \
         login_blocked_until = CASE \
           WHEN (CASE WHEN login_blocked_until IS NOT NULL AND login_blocked_until <= ?3 \
             THEN 1 ELSE failed_login_count + 1 END) >= ?1 THEN ?2 \
           ELSE NULL END, \
         failed_login_count = CASE \
           WHEN login_blocked_until IS NOT NULL AND login_blocked_until <= ?3 THEN 1 \
           ELSE failed_login_count + 1 END, \
         updated_at = ?3 WHERE id = ?4",
    )
    .bind(&[
        JsValue::from_f64(f64::from(LOGIN_FAILURE_LIMIT)),
        js_number(blocked_until),
        js_number(now),
        JsValue::from_str(user_id),
    ])?
    .run()
    .await?;
    Ok(())
}

async fn account_rate_allowed(env: &Env, scope: &str, email: &str) -> Result<bool> {
    let account_hash = sha256_b64(email.as_bytes()).await?;
    Ok(env
        .rate_limiter(AUTH_ACCOUNT_LIMITER_BINDING)?
        .limit(format!("{scope}:{account_hash}"))
        .await?
        .success)
}

async fn kdf_rate_allowed(env: &Env) -> Result<bool> {
    Ok(env
        .rate_limiter(AUTH_KDF_LIMITER_BINDING)?
        .limit("password-auth".to_owned())
        .await?
        .success)
}

async fn read_json<T: DeserializeOwned>(
    request: &mut Request,
) -> Result<std::result::Result<T, AuthProblem>> {
    let content_type = request.headers().get("content-type")?;
    if !content_type
        .as_deref()
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
    {
        return Ok(Err(AuthProblem::new(
            "unsupported_media_type",
            "Send a JSON request body.",
            415,
        )));
    }
    if request
        .headers()
        .get("content-length")?
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|size| size > MAX_JSON_BYTES)
    {
        return Ok(Err(body_too_large()));
    }

    let mut body = Vec::new();
    let mut stream = match request.stream() {
        Ok(stream) => stream,
        Err(worker::Error::RustError(message)) if message == "no body for request" => {
            return Ok(Err(invalid_json()));
        }
        Err(error) => return Err(error),
    };
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if body.len().saturating_add(chunk.len()) > MAX_JSON_BYTES {
            return Ok(Err(body_too_large()));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&body).map_err(|_| invalid_json()))
}

async fn pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>> {
    if !(PASSWORD_ITERATIONS..=2_400_000).contains(&iterations) || salt.len() != PASSWORD_SALT_BYTES
    {
        return Err(worker::Error::RustError(
            "stored password derivation parameters are invalid".into(),
        ));
    }
    let crypto = crypto()?;
    let subtle = crypto.subtle();
    let password_bytes = Uint8Array::from(password.as_bytes());
    let usages = Array::of1(&JsValue::from_str("deriveBits"));
    let key = worker::js_sys::futures::JsFuture::from(subtle.import_key_with_str(
        "raw",
        password_bytes.unchecked_ref::<Object>(),
        "PBKDF2",
        false,
        usages.as_ref(),
    )?)
    .await?
    .dyn_into::<web_sys::CryptoKey>()?;
    let salt_array = Uint8Array::from(salt);
    let params = web_sys::Pbkdf2Params::new_with_str(
        "PBKDF2",
        "SHA-256",
        iterations,
        salt_array.unchecked_ref::<Object>(),
    );
    let derived = worker::js_sys::futures::JsFuture::from(subtle.derive_bits_with_object(
        params.unchecked_ref::<Object>(),
        &key,
        (PASSWORD_HASH_BYTES * 8) as u32,
    )?)
    .await?
    .dyn_into::<ArrayBuffer>()?;
    Ok(Uint8Array::new(&derived).to_vec())
}

async fn sha256_b64(bytes: &[u8]) -> Result<String> {
    let digest = worker::js_sys::futures::JsFuture::from(
        crypto()?
            .subtle()
            .digest_with_str_and_u8_array("SHA-256", bytes)?,
    )
    .await?
    .dyn_into::<ArrayBuffer>()?;
    Ok(URL_SAFE_NO_PAD.encode(Uint8Array::new(&digest).to_vec()))
}

fn crypto() -> Result<web_sys::Crypto> {
    worker::js_sys::Reflect::get(&worker::js_sys::global(), &JsValue::from_str("crypto"))?
        .dyn_into::<web_sys::Crypto>()
        .map_err(Into::into)
}

fn random_bytes(size: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0_u8; size];
    crypto()?.get_random_values_with_u8_array(&mut bytes)?;
    Ok(bytes)
}

fn unix_seconds() -> i64 {
    (worker::js_sys::Date::now() / 1_000.0).floor() as i64
}

fn js_number(value: i64) -> JsValue {
    JsValue::from_f64(value as f64)
}

fn cookie_value(request: &Request, name: &str) -> Result<Option<String>> {
    Ok(request.headers().get("cookie")?.and_then(|header| {
        header.split(';').find_map(|part| {
            let (candidate, value) = part.trim().split_once('=')?;
            (candidate == name && !value.is_empty()).then(|| value.to_owned())
        })
    }))
}

fn normalized_email(input: &str) -> std::result::Result<String, AuthProblem> {
    let email = input.trim().to_lowercase();
    let valid = email.len() <= 254
        && email.len() >= 3
        && !email.bytes().any(|byte| byte.is_ascii_whitespace())
        && email.matches('@').count() == 1
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && domain.contains('.')
                && !domain.contains("..")
        });
    valid
        .then_some(email)
        .ok_or_else(|| AuthProblem::new("invalid_email", "Enter a valid email address.", 400))
}

fn bounded_display_name(input: &str) -> std::result::Result<String, AuthProblem> {
    let name = input.trim();
    let length = name.chars().count();
    (2..=80)
        .contains(&length)
        .then_some(())
        .filter(|_| !name.chars().any(char::is_control))
        .map(|()| name.to_owned())
        .ok_or_else(|| {
            AuthProblem::new(
                "invalid_display_name",
                "Display name must contain 2 to 80 visible characters.",
                400,
            )
        })
}

fn valid_new_password(password: &str) -> std::result::Result<(), AuthProblem> {
    let length = password.chars().count();
    if !(12..=128).contains(&length) || password.len() > 512 {
        return Err(AuthProblem::new(
            "invalid_password",
            "Password must contain 12 to 128 characters.",
            400,
        ));
    }
    Ok(())
}

fn bounded_invite_code(input: &str) -> std::result::Result<&str, AuthProblem> {
    let code = input.trim();
    (8..=128)
        .contains(&code.len())
        .then_some(code)
        .filter(|candidate| !candidate.bytes().any(|byte| byte.is_ascii_whitespace()))
        .ok_or_else(|| {
            AuthProblem::new(
                "invite_invalid",
                "The invite code is invalid, expired, revoked, or fully used.",
                400,
            )
        })
}

fn timing_safe_equal(left: &[u8], right: &[u8]) -> Result<bool> {
    let subtle = crypto()?.subtle();
    let function = Reflect::get(subtle.as_ref(), &JsValue::from_str("timingSafeEqual"))?
        .dyn_into::<Function>()?;
    let left = Uint8Array::from(left);
    let right = Uint8Array::from(right);
    let compared = if left.length() == right.length() {
        function.call2(subtle.as_ref(), left.as_ref(), right.as_ref())?
    } else {
        // Cloudflare's native helper requires equal lengths. Calling it even
        // for a mismatch avoids a secret-length early return; negating a
        // self-comparison deterministically yields false.
        let self_comparison = function.call2(subtle.as_ref(), left.as_ref(), left.as_ref())?;
        let equal = self_comparison.as_bool().ok_or_else(|| {
            worker::Error::RustError("crypto.subtle.timingSafeEqual returned a non-boolean".into())
        })?;
        JsValue::from_bool(!equal)
    };
    compared.as_bool().ok_or_else(|| {
        worker::Error::RustError("crypto.subtle.timingSafeEqual returned a non-boolean".into())
    })
}

fn session_user(user: &LoginUserRow) -> SessionUser {
    SessionUser {
        id: user.id.clone(),
        email: user.email_normalized.clone(),
        display_name: user.display_name.clone(),
        verification_level: user.verification_level,
        role: user.role.clone(),
    }
}

fn guest_session() -> SessionResponse {
    SessionResponse {
        authenticated: false,
        user: None,
    }
}

const fn invalid_credentials() -> AuthProblem {
    AuthProblem::new(
        "invalid_credentials",
        "The email address or password is incorrect.",
        401,
    )
}

const fn csrf_problem() -> AuthProblem {
    AuthProblem::new(
        "csrf_forbidden",
        "The request could not be verified. Refresh the page and try again.",
        403,
    )
}

const fn invalid_json() -> AuthProblem {
    AuthProblem::new("invalid_json", "Send a valid JSON request body.", 400)
}

const fn body_too_large() -> AuthProblem {
    AuthProblem::new(
        "body_too_large",
        "The request body exceeds the 16 KiB limit.",
        413,
    )
}

const fn rate_limited() -> AuthProblem {
    AuthProblem::new(
        "auth_rate_limited",
        "Too many authentication attempts. Please wait before trying again.",
        429,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalization_is_bounded_and_conservative() {
        assert_eq!(
            normalized_email("  Member@Example.COM ").expect("valid email"),
            "member@example.com"
        );
        assert!(normalized_email("missing-at.example.com").is_err());
        assert!(normalized_email("a@example.").is_err());
    }

    #[test]
    fn password_and_display_name_limits_count_unicode_safely() {
        assert!(valid_new_password("twelve-chars!").is_ok());
        assert!(valid_new_password(&"ā".repeat(128)).is_ok());
        assert!(valid_new_password("short").is_err());
        assert_eq!(bounded_display_name("  Sādhaka  ").unwrap(), "Sādhaka");
        assert!(bounded_display_name("A\nB").is_err());
    }

    #[test]
    fn session_cookies_keep_tokens_out_of_json_and_apply_secure_attributes() {
        let tokens = SessionTokens {
            session: "session-token".into(),
            csrf: "csrf-token".into(),
        };
        let reply = AuthReply::json(guest_session(), 200)
            .expect("serializable response")
            .with_session_cookies(&tokens);
        assert_eq!(reply.set_cookies.len(), 2);
        assert!(reply.set_cookies[0].contains("Secure; HttpOnly; SameSite=Lax"));
        assert!(reply.set_cookies[1].contains("Secure; SameSite=Lax"));
        assert!(!reply.body.to_string().contains("session-token"));
        assert!(!reply.body.to_string().contains("csrf-token"));
    }
}
