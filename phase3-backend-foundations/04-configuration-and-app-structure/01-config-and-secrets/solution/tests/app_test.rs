//! The configured CORS policy, exercised without a socket. These pass from the
//! start: they test the given `app`, not your functions.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use p3_04_01_config_and_secrets_solution::{app, Config};
use secrecy::SecretString;
use tower::ServiceExt;

fn config(origins: &[&str]) -> Config {
    Config {
        port: 3000,
        allowed_origins: origins.iter().map(|o| o.to_string()).collect(),
        debug: false,
        db_password: SecretString::from("pw"),
    }
}

async fn allow_origin_for(cfg: &Config, origin: &str) -> (StatusCode, Option<String>) {
    let req = Request::get("/")
        .header("origin", origin)
        .body(Body::empty())
        .unwrap();
    let res = app(cfg).unwrap().oneshot(req).await.unwrap();
    let header = res
        .headers()
        .get("access-control-allow-origin")
        .map(|v| v.to_str().unwrap().to_string());
    (res.status(), header)
}

#[tokio::test]
async fn a_configured_origin_is_vouched_for() {
    let cfg = config(&["https://anime.example.com"]);
    let (status, header) = allow_origin_for(&cfg, "https://anime.example.com").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(header.as_deref(), Some("https://anime.example.com"));
}

#[tokio::test]
async fn an_unlisted_origin_gets_a_response_but_no_vouching() {
    let cfg = config(&["https://anime.example.com"]);
    let (status, header) = allow_origin_for(&cfg, "https://evil.example").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(header, None);
}

#[test]
fn an_origin_that_is_not_a_header_value_is_a_config_error() {
    let err = app(&config(&["bad\norigin"])).unwrap_err();
    assert!(err.to_string().contains("APP_ALLOWED_ORIGINS"), "{err}");
}
