//! Issuing and verifying JWTs, then gating an `axum` route behind a
//! `require_auth` middleware — the JWT-verifying sibling of
//! `capstone-taskforge/taskforge-api/src/auth.rs`'s `require_bearer_token`,
//! which just string-compares against one static shared secret. See
//! `README.md` for the theory (what a JWT actually is, why it's signed but
//! NOT encrypted) before touching the `todo!()`s below. Tests live in
//! `tests/jwt_test.rs`, not inline — this is a "mini-project" lesson in the
//! sense `docs/conventions.md` describes (a router, a middleware, a
//! protected handler working together), so its tests exercise only the
//! crate's public surface, same as `taskforge-api`'s and the anime catalog
//! lesson's `tests/api_test.rs`.

use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

// Note: `jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header,
// Validation}` and `std::time::{SystemTime, UNIX_EPOCH}` are deliberately
// NOT imported here yet — they're only needed inside the two `todo!()`
// bodies below, so importing them now would trigger `unused_imports`
// warnings until you actually write that code. Add them yourself as part
// of filling in `issue_token` and `require_auth`.

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

/// Issues a signed JWT for `user_id`, valid for one hour from now.
pub fn issue_token(user_id: &str, secret: &str) -> String {
    todo!(
        "add `use jsonwebtoken::{{encode, EncodingKey, Header}};` and `use std::time::{{SystemTime, \
         UNIX_EPOCH}};` at the top of this file; compute now-as-seconds with \
         SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(); add 3600 (one hour) to \
         get the expiration timestamp, cast to usize for exp; build a Claims value with sub: \
         user_id.to_string() and that exp; call encode(&Header::default(), &claims, \
         &EncodingKey::from_secret(secret.as_bytes())), which returns a Result<String, _> — \
         .unwrap() it (encoding a valid Claims struct should never fail) and return the token"
    )
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
    todo!(
        "add `use jsonwebtoken::{{decode, DecodingKey, Validation}};` at the top of this file; \
         pull the Authorization header: request.headers().get(axum::http::header::AUTHORIZATION) \
         .and_then(|v| v.to_str().ok()); if it's Some(value) and value.strip_prefix(\"Bearer \") \
         gives you Some(token), keep going with `token` — otherwise return \
         Err(StatusCode::UNAUTHORIZED) immediately; call decode::<Claims>(token, \
         &DecodingKey::from_secret(secret.as_bytes()), &Validation::default()), which returns a \
         Result — on Err(_) (bad signature, malformed token, OR expired — decode checks exp for \
         you) return Err(StatusCode::UNAUTHORIZED); on Ok(data), insert \
         AuthUser(data.claims.sub) into request.extensions_mut(), then return \
         Ok(next.run(request).await)"
    )
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
