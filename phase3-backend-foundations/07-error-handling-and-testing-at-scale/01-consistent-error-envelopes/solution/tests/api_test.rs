//! Drives the whole stack — router, extractors, handlers, store — through
//! `tower::ServiceExt::oneshot`, the same technique as `02-axum-and-rest-
//! api-design/02-anime-catalog-crud-in-memory`. This is a "mini-project"
//! lesson per `docs/conventions.md`, so tests live here in `tests/` rather
//! than inline `#[cfg(test)]` — they only see this crate's `pub` surface,
//! exactly like a real consumer (or a frontend developer reading the error
//! envelope) would.
//!
//! Every error-case assertion here checks *two* things on purpose: the HTTP
//! status code, AND the shape of the JSON error body. Checking only the
//! status code would let a handler regress to some other ad-hoc error body
//! shape without any test noticing — the whole point of a consistent
//! envelope is that its shape is itself a contract worth testing.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_07_01_consistent_error_envelopes_solution::{app, WidgetStore};

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn fresh_app() -> axum::Router {
    app(Arc::new(WidgetStore::default()))
}

#[tokio::test]
async fn create_then_get_a_widget() {
    let store = Arc::new(WidgetStore::default());

    let response = app(store.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/widgets")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"name": "gizmo", "quantity": 12}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    assert_eq!(created["name"], "gizmo");
    assert_eq!(created["quantity"], 12);
    let id = created["id"].as_u64().unwrap();

    let response = app(store)
        .oneshot(
            Request::builder()
                .uri(format!("/widgets/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = json_body(response).await;
    assert_eq!(fetched["name"], "gizmo");
}

#[tokio::test]
async fn get_missing_widget_returns_404_with_a_not_found_envelope() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .uri("/widgets/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["error"]["code"], "not_found");
    assert!(body["error"]["message"].as_str().unwrap().contains("9999"));
}

#[tokio::test]
async fn create_with_empty_name_returns_400_with_a_validation_envelope() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/widgets")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "", "quantity": 1}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(body["error"]["message"].as_str().unwrap().contains("name"));
}

#[tokio::test]
async fn create_with_out_of_range_quantity_returns_400_with_a_validation_envelope() {
    let response = fresh_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/widgets")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"name": "gizmo", "quantity": 0}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["error"]["code"], "validation_failed");
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("quantity"));
}

#[tokio::test]
async fn every_error_response_shares_the_same_envelope_shape() {
    // Two completely different failure modes (404 vs 400) still produce a
    // body with exactly the same two top-level keys under "error" — that
    // consistency is the thing this lesson builds.
    let not_found = fresh_app()
        .oneshot(
            Request::builder()
                .uri("/widgets/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let not_found_body = json_body(not_found).await;

    let invalid = fresh_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/widgets")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "", "quantity": 1}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let invalid_body = json_body(invalid).await;

    for body in [&not_found_body, &invalid_body] {
        let error = body["error"].as_object().unwrap();
        let mut keys: Vec<&str> = error.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["code", "message"]);
    }
}
