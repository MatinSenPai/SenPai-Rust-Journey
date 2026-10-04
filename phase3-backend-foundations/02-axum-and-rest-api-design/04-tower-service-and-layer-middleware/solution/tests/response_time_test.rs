use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use tower::{service_fn, Layer, ServiceExt};

use p3_02_04_tower_service_and_layer_middleware_solution::ResponseTimeLayer;

fn get_request(uri: &str) -> Request {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn header_value(response: &Response) -> String {
    let value = response
        .headers()
        .get("x-response-time-ms")
        .expect("the response has no x-response-time-ms header");
    value.to_str().unwrap().to_string()
}

#[tokio::test]
async fn adds_the_header_as_plain_decimal_digits() {
    let app = Router::new()
        .route("/", get(|| async { "hello" }))
        .layer(ResponseTimeLayer);

    let response = app.oneshot(get_request("/")).await.unwrap();

    let value = header_value(&response);
    assert!(!value.is_empty());
    assert!(
        value.chars().all(|c| c.is_ascii_digit()),
        "expected only digits, got {value:?}"
    );
}

#[tokio::test]
async fn leaves_status_and_body_unchanged() {
    let app = Router::new()
        .route("/", get(|| async { (StatusCode::CREATED, "made") }))
        .layer(ResponseTimeLayer);

    let response = app.oneshot(get_request("/")).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"made");
}

#[tokio::test]
async fn times_the_whole_handler_not_just_the_call() {
    let app = Router::new()
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                "done"
            }),
        )
        .layer(ResponseTimeLayer);

    let response = app.oneshot(get_request("/slow")).await.unwrap();

    let millis: u64 = header_value(&response).parse().unwrap();
    assert!(millis >= 20, "a 20 ms handler was timed at {millis} ms");
}

#[tokio::test]
async fn replaces_an_existing_header_instead_of_adding_a_second_one() {
    let app = Router::new()
        .route(
            "/",
            get(|| async { ([("x-response-time-ms", "999")], "hi").into_response() }),
        )
        .layer(ResponseTimeLayer);

    let response = app.oneshot(get_request("/")).await.unwrap();

    let all: Vec<_> = response
        .headers()
        .get_all("x-response-time-ms")
        .iter()
        .collect();
    assert_eq!(all.len(), 1);
    assert_ne!(all[0], "999");
}

#[tokio::test]
async fn passes_an_inner_error_through_unchanged() {
    let failing = service_fn(|_request: Request| async { Err::<Response, &str>("boom") });
    let service = ResponseTimeLayer.layer(failing);

    let result = service.oneshot(get_request("/")).await;

    assert_eq!(result.unwrap_err(), "boom");
}
