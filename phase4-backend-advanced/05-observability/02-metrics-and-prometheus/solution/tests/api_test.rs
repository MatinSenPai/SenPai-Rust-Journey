//! Integration tests against `build_router`'s public surface — this is a
//! "mini-project" lesson (a small axum app), so tests live in `tests/`
//! rather than inline, exactly the same split `capstone-taskforge` and the
//! CRUD API lessons in Phase 3 use. Every test calls `build_router()` fresh
//! (a new, empty `widgets` map each time), but note the Prometheus
//! *recorder* itself is process-global (`metrics_handle`'s `OnceLock`) —
//! that's exactly why the metrics assertions below check that expected
//! metric names/labels appear in `/metrics`'s body, rather than asserting
//! an exact counter value: other tests in this file may have already
//! incremented the same counters before this test's request runs.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use p4_05_02_metrics_and_prometheus_solution::build_router;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn create_request(name: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/widgets")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "name": name }).to_string()))
        .unwrap()
}

fn get_request(uri: String) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn creating_a_widget_returns_it_with_an_id() {
    let response = build_router()
        .oneshot(create_request("gizmo"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["name"], "gizmo");
    assert!(body["id"].as_u64().is_some());
}

#[tokio::test]
async fn getting_a_missing_widget_returns_404() {
    let response = build_router()
        .oneshot(get_request("/widgets/999999".to_string()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn getting_a_widget_after_creating_it_finds_it() {
    let router = build_router();

    let create_response = router
        .clone()
        .oneshot(create_request("widget-a"))
        .await
        .unwrap();
    let created = body_json(create_response).await;
    let id = created["id"].as_u64().unwrap();

    let get_response = router
        .oneshot(get_request(format!("/widgets/{id}")))
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = body_json(get_response).await;
    assert_eq!(body["name"], "widget-a");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn metrics_endpoint_exposes_the_expected_counters_and_histograms() {
    let router = build_router();

    // One create (a hit-producing lookup target), one hit lookup, one miss
    // lookup — enough traffic to guarantee every metric name and label
    // this lesson cares about has been recorded at least once.
    let create_response = router
        .clone()
        .oneshot(create_request("metrics-probe"))
        .await
        .unwrap();
    let created = body_json(create_response).await;
    let id = created["id"].as_u64().unwrap();

    router
        .clone()
        .oneshot(get_request(format!("/widgets/{id}")))
        .await
        .unwrap();
    router
        .clone()
        .oneshot(get_request("/widgets/999999".to_string()))
        .await
        .unwrap();

    let metrics_response = router
        .oneshot(get_request("/metrics".to_string()))
        .await
        .unwrap();
    assert_eq!(metrics_response.status(), StatusCode::OK);

    let body = body_text(metrics_response).await;
    assert!(
        body.contains("widgets_created_total"),
        "missing counter in:\n{body}"
    );
    assert!(
        body.contains("widget_create_duration_seconds"),
        "missing histogram in:\n{body}"
    );
    assert!(
        body.contains("widget_lookups_total"),
        "missing counter in:\n{body}"
    );
    assert!(
        body.contains("widget_lookup_duration_seconds"),
        "missing histogram in:\n{body}"
    );
    assert!(
        body.contains(r#"result="hit""#),
        "missing hit-labeled series in:\n{body}"
    );
    assert!(
        body.contains(r#"result="miss""#),
        "missing miss-labeled series in:\n{body}"
    );
}
