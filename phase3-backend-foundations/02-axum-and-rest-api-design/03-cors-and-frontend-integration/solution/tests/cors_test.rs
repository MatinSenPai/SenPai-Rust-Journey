//! Same tests as the exercise crate, pointed at the solution's package
//! name — see the starter's `tests/cors_test.rs` for the full commentary.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::response::Response;
use tower::ServiceExt;

use p3_02_03_cors_and_frontend_integration_solution::{app, dev_cors, prod_cors};

const FRONTEND: &str = "https://anime.example.com";

fn preflight(origin: &str) -> Request<Body> {
    Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap()
}

fn get_with_origin(origin: &str) -> Request<Body> {
    Request::builder()
        .uri("/anime")
        .header("origin", origin)
        .body(Body::empty())
        .unwrap()
}

fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
    response
        .headers()
        .get(name)
        .map(|value| value.to_str().unwrap())
}

#[tokio::test]
async fn dev_preflight_vouches_for_any_origin() {
    let response = app(dev_cors())
        .oneshot(preflight("http://localhost:5173"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), Some("*"));
    assert_eq!(header(&response, "access-control-allow-methods"), Some("*"));
    assert_eq!(header(&response, "access-control-allow-headers"), Some("*"));
}

#[tokio::test]
async fn preflights_are_answered_by_the_layer_not_a_handler() {
    let response = app(dev_cors())
        .oneshot(preflight("http://localhost:5173"))
        .await
        .unwrap();

    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(bytes.is_empty());
}

#[tokio::test]
async fn dev_simple_request_carries_the_wildcard() {
    let response = app(dev_cors())
        .oneshot(get_with_origin("http://localhost:5173"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), Some("*"));
}

#[tokio::test]
async fn prod_preflight_from_the_allowed_origin_is_vouched_for() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(preflight(FRONTEND))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, "access-control-allow-origin"),
        Some(FRONTEND)
    );

    let methods = header(&response, "access-control-allow-methods").unwrap();
    assert!(methods.contains("GET"), "allow-methods was {methods:?}");
    assert!(methods.contains("POST"), "allow-methods was {methods:?}");

    let headers = header(&response, "access-control-allow-headers")
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        headers.contains("content-type"),
        "allow-headers was {headers:?}"
    );
}

#[tokio::test]
async fn prod_preflight_from_an_unknown_origin_is_not_vouched_for() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(preflight("https://evil.example.com"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), None);
}

#[tokio::test]
async fn prod_simple_request_echoes_exactly_the_allowed_origin() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(get_with_origin(FRONTEND))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, "access-control-allow-origin"),
        Some(FRONTEND)
    );
}
