//! A real server on 127.0.0.1:3190, the quick version: the real clock, no
//! typed errors, a bare `401`. `POST /login` hands out a token for any name
//! (a stand-in: 3.7.1 is where a real login checks a password hash),
//! `GET /whoami` needs one, `GET /health` is public.

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use p3_07_03_jwt_and_tower_middleware::{system_now, AuthUser, Claims};
use serde::Deserialize;
use serde_json::json;

// A demo secret. A real one is 32+ random bytes read from configuration.
const SECRET: &str = "demo-secret-for-3-7-3";

#[derive(Deserialize)]
struct Login {
    user: String,
}

async fn login(Json(body): Json<Login>) -> Json<serde_json::Value> {
    let now = system_now();
    let claims = Claims {
        sub: body.user,
        iat: now,
        exp: now + 3600,
    };
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    Json(json!({ "token": encode(&Header::default(), &claims, &key).unwrap() }))
}

// Pulls the token out of `Authorization: Bearer <token>`. Quick and loose: the
// lesson's `bearer_token` exercise is the strict version.
fn bearer(request: &Request) -> Option<&str> {
    let value = request.headers().get(header::AUTHORIZATION)?;
    value.to_str().ok()?.strip_prefix("Bearer ")
}

async fn require_auth(
    State(secret): State<&'static str>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let token = bearer(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    let data = decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(AuthUser(data.claims.sub));
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/whoami", get(p3_07_03_jwt_and_tower_middleware::whoami))
        .route_layer(from_fn_with_state(SECRET, require_auth))
        .route("/health", get(|| async { "ok" }))
        .route("/login", post(login));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3190")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3190");
    axum::serve(listener, router).await.unwrap();
}
