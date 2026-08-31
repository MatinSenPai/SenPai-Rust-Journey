//! Issuing and verifying JWTs, then gating an `axum` route behind a
//! `require_auth` middleware — the JWT-verifying sibling of
//! `capstone-taskforge/taskforge-api/src/auth.rs`'s `require_bearer_token`,
//! which just string-compares against one static shared secret. See
//! `README.md` for the theory (what a JWT actually is, why it's signed but
//! NOT encrypted). Tests live in `tests/jwt_test.rs`, not inline — this is
//! a "mini-project" lesson in the sense `docs/conventions.md` describes (a
//! router, a middleware, a protected handler working together), so its
//! tests exercise only the crate's public surface, same as
//! `taskforge-api`'s and the anime catalog lesson's `tests/api_test.rs`.

use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// The JWT payload ("claims"). `sub` ("subject") is the standard claim name
/// for "who is this token about" — here, a user id as a string. `exp`
/// ("expiration") is a Unix timestamp in seconds; `jsonwebtoken::decode`
/// checks it automatically and rejects an expired token on its own, no
/// manual comparison needed on your end.
///
/// **Never put secrets in here.** A JWT's payload is base64url-*encoded*,
/// not encrypted — anyone holding the token string can decode and read
/// every field with nothing more than a text editor (try it: paste any JWT
/// into <https://jwt.io> and watch the payload appear, no key required).
/// The signature only proves the payload hasn't been *tampered with* since
/// your server issued it; it proves nothing about who is allowed to *read*
/// it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

const ONE_HOUR_IN_SECONDS: u64 = 3600;

/// Issues a signed JWT for `user_id`, valid for one hour from now.
pub fn issue_token(user_id: &str, secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + ONE_HOUR_IN_SECONDS) as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("encoding a valid Claims struct should never fail")
}

/// The user id `require_auth` decoded from a validated token, inserted into
/// the request's extensions so downstream handlers can pull it back out
/// with the `Extension<AuthUser>` extractor instead of re-parsing (and
/// re-verifying) the header themselves.
#[derive(Debug, Clone)]
pub struct AuthUser(pub String);

/// Extracts `Authorization: Bearer <token>`, validates it against `secret`,
/// and either lets the request through (with `AuthUser` inserted into its
/// extensions) or returns `401 Unauthorized` — for a missing header, a
/// malformed token, a token signed with the wrong secret, AND an expired
/// token alike. A caller checking whether a request is authorized never
/// needs to distinguish those cases; they all mean the same thing: reject
/// it.
pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthUser(data.claims.sub));

    Ok(next.run(request).await)
}

/// A tiny protected handler used by `tests/jwt_test.rs`: echoes back
/// whichever user id `require_auth` decoded and stashed on the request.
pub async fn whoami(Extension(user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.0 }))
}

/// Wires one protected route (`GET /whoami`) behind `require_auth`. Any
/// request without a valid `Authorization: Bearer <token>` header never
/// reaches `whoami` at all — `require_auth` short-circuits with `401`
/// first, the same "middleware runs before the handler, and can refuse to
/// call it" shape as `taskforge-api::build_router`'s `route_layer`.
pub fn app(secret: String) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(secret, require_auth))
}
