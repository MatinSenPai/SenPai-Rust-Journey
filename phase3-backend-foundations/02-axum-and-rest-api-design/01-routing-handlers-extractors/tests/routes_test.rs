use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for `Router::oneshot`

use p3_02_01_routing_handlers_extractors::{app, AppState};

async fn body_string(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn root_returns_hello_world() {
    let response = app(AppState::default())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_string(response).await, "Hello, world!");
}

#[tokio::test]
async fn greet_uses_the_path_segment() {
    let response = app(AppState::default())
        .oneshot(
            Request::builder()
                .uri("/greet/senpai")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_string(response).await, "Hello, senpai!");
}

#[tokio::test]
async fn echo_round_trips_json_and_reports_length() {
    let response = app(AppState::default())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/echo")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"hi"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let text = body_string(response).await;
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["message"], "hi");
    assert_eq!(json["length"], 2);
}

#[tokio::test]
async fn echo_rejects_a_malformed_json_body_before_the_handler_runs() {
    let response = app(AppState::default())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/echo")
                .header("content-type", "application/json")
                .body(Body::from("not json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn counter_starts_at_zero_and_increments() {
    let state = AppState::default();

    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .uri("/counter")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_string(response).await;
    assert_eq!(text, r#"{"count":0}"#);

    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/counter/increment")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_string(response).await;
    assert_eq!(text, r#"{"count":1}"#);

    let response = app(state)
        .oneshot(
            Request::builder()
                .uri("/counter")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let text = body_string(response).await;
    assert_eq!(text, r#"{"count":1}"#);
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let response = app(AppState::default())
        .oneshot(Request::builder().uri("/nope").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
