//! CORS without a browser. A preflight is nothing more than an `OPTIONS`
//! request carrying `Origin` and `Access-Control-Request-*` headers, so
//! `tower::ServiceExt::oneshot` (the same technique as 3.2.1) can fabricate
//! one, and every test below asserts on the `access-control-*` response
//! headers directly.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use tower::ServiceExt;

use p3_02_05_cors_and_frontend_integration_solution::{
    app, dev_cors, prod_cors, prod_cors_from_list,
};

const FRONTEND: &str = "https://anime.example.com";
const LOCAL_DEV: &str = "http://localhost:5173";
const EVIL: &str = "https://evil.example.com";

/// Exactly what a browser sends before a cross-origin `POST` carrying the
/// request headers named in `request_headers`.
fn preflight(origin: &str, request_headers: &str) -> Request<Body> {
    Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", request_headers)
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

// ---------------------------------------------------------------- dev_cors

#[tokio::test]
async fn dev_preflight_vouches_for_everything_with_a_wildcard() {
    let response = app(dev_cors())
        .oneshot(preflight(LOCAL_DEV, "content-type"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), Some("*"));
    assert_eq!(header(&response, "access-control-allow-methods"), Some("*"));
    assert_eq!(header(&response, "access-control-allow-headers"), Some("*"));
    assert_eq!(header(&response, "access-control-allow-credentials"), None);
}

#[tokio::test]
async fn dev_simple_request_carries_the_wildcard() {
    let response = app(dev_cors())
        .oneshot(get_with_origin(LOCAL_DEV))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), Some("*"));
}

async fn count_and_create(State(hits): State<Arc<AtomicUsize>>) -> StatusCode {
    hits.fetch_add(1, Ordering::SeqCst);
    StatusCode::CREATED
}

#[tokio::test]
async fn a_preflight_never_reaches_the_handler() {
    // The CorsLayer is a Layer like the one you wrote in 3.2.4, and on a
    // preflight it short-circuits: it answers without calling what it wraps.
    let hits = Arc::new(AtomicUsize::new(0));
    let counted = Router::new()
        .route("/anime", post(count_and_create))
        .with_state(hits.clone())
        .layer(dev_cors());

    let response = counted
        .clone()
        .oneshot(preflight(LOCAL_DEV, "content-type"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(body.is_empty(), "a preflight response has no body");
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "the handler ran on a preflight"
    );

    // The real request, which the preflight approved, does reach it.
    let real = Request::builder()
        .method("POST")
        .uri("/anime")
        .header("origin", LOCAL_DEV)
        .body(Body::empty())
        .unwrap();
    let response = counted.oneshot(real).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

// --------------------------------------------------------------- prod_cors

#[tokio::test]
async fn prod_preflight_from_the_allowed_origin_is_vouched_for() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(preflight(FRONTEND, "content-type"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        header(&response, "access-control-allow-origin"),
        Some(FRONTEND)
    );
    assert_eq!(header(&response, "access-control-allow-credentials"), None);

    let methods = header(&response, "access-control-allow-methods").unwrap();
    assert!(methods.contains("GET"), "allow-methods was {methods:?}");
    assert!(methods.contains("POST"), "allow-methods was {methods:?}");
    assert!(!methods.contains("DELETE"), "allow-methods was {methods:?}");

    let headers = header(&response, "access-control-allow-headers")
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        headers.contains("content-type"),
        "allow-headers was {headers:?}"
    );
}

#[tokio::test]
async fn prod_preflight_does_not_vouch_for_other_request_headers() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(preflight(FRONTEND, "content-type, authorization"))
        .await
        .unwrap();

    // Still 200, still vouched for as an origin: the server never "rejects"
    // a header, it lists what it allows and the browser compares.
    assert_eq!(response.status(), StatusCode::OK);
    let headers = header(&response, "access-control-allow-headers")
        .unwrap()
        .to_ascii_lowercase();
    assert!(
        !headers.contains("authorization") && headers != "*",
        "allow-headers was {headers:?}"
    );
}

#[tokio::test]
async fn prod_preflight_from_an_unknown_origin_is_not_vouched_for() {
    let response = app(prod_cors(FRONTEND))
        .oneshot(preflight(EVIL, "content-type"))
        .await
        .unwrap();

    // The server never "blocks": it answers 200 and simply declines to
    // vouch. No allow-origin header, so the *browser* withholds the
    // response from the page.
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

// ----------------------------------------------- prod_cors_from_list (Build)

#[tokio::test]
async fn list_vouches_for_every_listed_origin_and_nobody_else() {
    let origins = format!(" {FRONTEND} ,{LOCAL_DEV},");

    for origin in [FRONTEND, LOCAL_DEV] {
        let response = app(prod_cors_from_list(&origins))
            .oneshot(preflight(origin, "content-type"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            header(&response, "access-control-allow-origin"),
            Some(origin)
        );
    }

    let response = app(prod_cors_from_list(&origins))
        .oneshot(preflight(EVIL, "content-type"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(header(&response, "access-control-allow-origin"), None);
}

#[tokio::test]
async fn list_keeps_prod_methods_and_headers() {
    let response = app(prod_cors_from_list(FRONTEND))
        .oneshot(preflight(FRONTEND, "content-type, authorization"))
        .await
        .unwrap();

    let methods = header(&response, "access-control-allow-methods").unwrap();
    assert!(methods.contains("GET") && methods.contains("POST"));
    let headers = header(&response, "access-control-allow-headers")
        .unwrap()
        .to_ascii_lowercase();
    assert!(headers.contains("content-type") && !headers.contains("authorization"));
    assert_eq!(header(&response, "access-control-allow-credentials"), None);
}

#[test]
#[should_panic(expected = "https://anime.example.com/")]
fn list_refuses_an_origin_with_a_trailing_slash() {
    let _ = prod_cors_from_list("http://localhost:5173, https://anime.example.com/");
}
