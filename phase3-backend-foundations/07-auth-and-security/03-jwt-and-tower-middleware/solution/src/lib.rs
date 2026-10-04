//! Solution for 3.7.3 — JWTs and `tower` middleware.
//!
//! One crate, one story: issue a signed token, verify it against a clock you
//! inject, and gate routes behind `axum` middleware that hands the verified
//! identity to handlers through request extensions.

use std::fmt;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Extension, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------ the given

/// The JWT payload. `sub` is who the token is about, `iat` is when it was
/// issued and `exp` when it stops being valid, both in whole seconds since the
/// Unix epoch.
///
/// A JWT payload is only base64url-encoded, not encrypted: anyone holding the
/// token can read every field. Never put a secret in here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

/// A source of "now", in whole seconds since the Unix epoch. Tests pass a
/// closure that returns a fixed number, so no test depends on the real clock.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// The real clock: whole seconds since the Unix epoch.
pub fn system_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_secs()
}

/// Everything token issuing and verifying need. Cheap to clone: `axum`
/// clones it for every request.
#[derive(Clone)]
pub struct JwtConfig {
    /// The HMAC secret. Real secrets are long and random (32+ bytes) and come
    /// from configuration, never from source code.
    pub secret: String,
    /// How long a new token lives, in seconds.
    pub ttl_secs: u64,
    /// How many seconds past `exp` a token is still accepted, to absorb
    /// clock skew between machines.
    pub leeway_secs: u64,
    /// Where "now" comes from.
    pub clock: Clock,
}

impl JwtConfig {
    /// A config with a one-hour lifetime, 30 seconds of leeway and the real
    /// clock.
    pub fn new(secret: &str) -> Self {
        JwtConfig {
            secret: secret.to_string(),
            ttl_secs: 3600,
            leeway_secs: 30,
            clock: Arc::new(system_now),
        }
    }

    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    pub fn with_leeway(mut self, leeway_secs: u64) -> Self {
        self.leeway_secs = leeway_secs;
        self
    }

    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }
}

// A derived `Debug` would print the secret into every log line that formats
// the config. This one leaves it out.
impl fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtConfig")
            .field("secret", &"<redacted>")
            .field("ttl_secs", &self.ttl_secs)
            .field("leeway_secs", &self.leeway_secs)
            .finish_non_exhaustive()
    }
}

/// Why a request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// No usable `Authorization: Bearer ...` credentials on the request.
    Missing,
    /// Not a JWT this server can read: wrong shape, bad JSON, a missing
    /// claim, or a header that names no known algorithm.
    Malformed,
    /// Well-formed, but the signature does not match the claims and secret.
    BadSignature,
    /// Signed with an algorithm other than HS256.
    WrongAlgorithm,
    /// Genuine and well-formed, but past `exp` plus leeway.
    Expired,
}

impl IntoResponse for AuthError {
    /// `401 Unauthorized`, a `WWW-Authenticate: Bearer` header, and a JSON
    /// body `{"error": "<code>"}` where the code is `missing_token`,
    /// `token_expired`, or `invalid_token` (for every other variant).
    fn into_response(self) -> Response {
        let code = match self {
            AuthError::Missing => "missing_token",
            AuthError::Expired => "token_expired",
            _ => "invalid_token",
        };
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"))],
            Json(serde_json::json!({ "error": code })),
        )
            .into_response()
    }
}

/// The identity `require_auth` verified, stored in the request's extensions
/// for handlers further in.
#[derive(Debug, Clone)]
pub struct AuthUser(pub String);

/// Echoes the verified identity back.
pub async fn whoami(Extension(user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.0 }))
}

/// A handler for the admin route of [`admin_app`].
pub async fn admin_ping() -> &'static str {
    "pong"
}

/// `GET /whoami` behind [`require_auth`]. `GET /health` is registered after
/// the layer, so it is public: a layer only wraps the routes added before it.
pub fn app(config: JwtConfig) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(config, require_auth))
        .route("/health", get(|| async { "ok" }))
}

// ---------------------------------------------------------------- the ladder

/// Pulls the token out of an `Authorization` header value.
///
/// The value must be the scheme `Bearer` (letters in any case), exactly one
/// space, and then a non-empty token that contains no whitespace. Returns the
/// token. Anything else returns `None`: `"Bearer"`, `"Bearer "`,
/// `"Basic abc"`, `"abc"`, `"Bearer  abc"` (two spaces), `"Bearer a b"` and
/// the empty string.
pub fn bearer_token(header_value: &str) -> Option<&str> {
    let (scheme, token) = header_value.split_once(' ')?;
    let plain = !token.is_empty() && !token.contains(char::is_whitespace);
    (scheme.eq_ignore_ascii_case("Bearer") && plain).then_some(token)
}

/// Issues a signed HS256 token for `user_id`.
///
/// The claims are `sub = user_id`, `iat` = the config clock's current time,
/// and `exp` = `iat` + `ttl_secs`. The result is the usual compact
/// `header.payload.signature` string, signed with `config.secret`.
pub fn issue_token(config: &JwtConfig, user_id: &str) -> String {
    let now = (config.clock)();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + config.ttl_secs,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .expect("HS256 signing with a byte secret cannot fail")
}

/// Verifies `token` and returns its claims.
///
/// Only HS256 is accepted. The token must carry `sub`, `iat` and `exp`. The
/// outcomes, checked in this order:
///
/// 1. a header naming a different supported algorithm (HS384, HS512, RS256,
///    ...) is `WrongAlgorithm`;
/// 2. a signature that does not match is `BadSignature`;
/// 3. any other failure (not three dot-separated base64url parts, invalid
///    JSON, a missing claim, a header whose `alg` is not an algorithm this
///    library knows, such as `none`) is `Malformed`;
/// 4. a token with `now > exp + leeway_secs` is `Expired`, where `now` comes
///    from `config.clock`. At exactly `exp + leeway_secs` it is still valid.
pub fn verify_token(config: &JwtConfig, token: &str) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    // The library's own expiry check reads the system clock, which a test
    // cannot control, so it is off here and step 4 does the comparison.
    validation.validate_exp = false;
    validation.set_required_spec_claims(&["exp", "sub"]);

    let key = DecodingKey::from_secret(config.secret.as_bytes());
    let claims = decode::<Claims>(token, &key, &validation)
        .map_err(|error| match error.kind() {
            ErrorKind::InvalidAlgorithm => AuthError::WrongAlgorithm,
            ErrorKind::InvalidSignature => AuthError::BadSignature,
            _ => AuthError::Malformed,
        })?
        .claims;

    if (config.clock)() > claims.exp.saturating_add(config.leeway_secs) {
        return Err(AuthError::Expired);
    }
    Ok(claims)
}

/// Middleware: lets a request through only if it carries a valid token.
///
/// A request with no `Authorization` header, or one that is not a `Bearer`
/// credential (see [`bearer_token`]), is refused with `AuthError::Missing`.
/// Otherwise the token goes through [`verify_token`] and its error, if any,
/// is returned as is. On success the verified subject is stored as an
/// [`AuthUser`] in the request's extensions, so that a handler can take
/// `Extension<AuthUser>`, and the request continues to the next layer.
pub async fn require_auth(
    State(config): State<JwtConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_token)
        .ok_or(AuthError::Missing)?;
    let claims = verify_token(&config, token)?;
    request.extensions_mut().insert(AuthUser(claims.sub));
    Ok(next.run(request).await)
}

async fn require_subject(
    State(admin): State<Arc<str>>,
    Extension(user): Extension<AuthUser>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if user.0 == *admin {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Builds a router with two routes, both behind [`require_auth`]:
///
/// - `GET /whoami` ([`whoami`]) for any valid token;
/// - `GET /admin` ([`admin_ping`]) only when the token's subject is exactly
///   the `admin` argument. A valid token for any other subject gets
///   `403 Forbidden`.
///
/// A request with no valid token gets the `401` from [`require_auth`] on
/// both routes, even if it would also have failed the admin check.
pub fn admin_app(config: JwtConfig, admin: &str) -> Router {
    Router::new()
        .route("/admin", get(admin_ping))
        .route_layer(from_fn_with_state(Arc::<str>::from(admin), require_subject))
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(config, require_auth))
}
