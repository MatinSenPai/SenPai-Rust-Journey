//! A real server on 127.0.0.1:3191 built from YOUR finished ladder:
//! `admin_app` (and through it `require_auth`, `verify_token`, `issue_token`).
//! `POST /login` hands out a token for any name; the admin is `matin`.
//!
//! Run it only after the Implement and Build rungs pass. On the skeleton it
//! builds, and every request panics at the first `todo!()`.

use axum::routing::post;
use axum::{Json, Router};
use p3_07_03_jwt_and_tower_middleware::{admin_app, issue_token, JwtConfig};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Login {
    user: String,
}

#[tokio::main]
async fn main() {
    // A demo secret. A real one is 32+ random bytes read from configuration.
    let config = JwtConfig::new("demo-secret-for-3-7-3");
    let login_config = config.clone();
    let login = move |Json(body): Json<Login>| async move {
        Json(json!({ "token": issue_token(&login_config, &body.user) }))
    };

    let router = Router::new()
        .route("/login", post(login))
        .merge(admin_app(config, "matin"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3191")
        .await
        .unwrap();
    println!("listening on http://127.0.0.1:3191");
    axum::serve(listener, router).await.unwrap();
}
