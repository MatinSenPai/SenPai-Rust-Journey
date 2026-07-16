use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use taskforge_api::build_router;
use taskforge_core::JobStore;
use taskforge_storage::InMemoryJobStore;
use tower::ServiceExt;

const TOKEN: &str = "test-token";

fn router() -> axum::Router {
    let store: Arc<dyn JobStore> = Arc::new(InMemoryJobStore::new());
    build_router(store, TOKEN.to_string())
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn authed_request(method: &str, uri: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .unwrap()
}

#[tokio::test]
async fn health_check_needs_no_auth() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn jobs_routes_reject_missing_auth() {
    let response = router()
        .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn jobs_routes_reject_wrong_token() {
    let request = Request::builder()
        .uri("/jobs")
        .header(header::AUTHORIZATION, "Bearer wrong-token")
        .body(Body::empty())
        .unwrap();
    let response = router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn enqueue_then_get_round_trip() {
    let app = router();

    let enqueue_body = json!({"job_type": "send_email", "payload": {"to": "a@b.com"}}).to_string();
    let response = app
        .clone()
        .oneshot(authed_request("POST", "/jobs", Body::from(enqueue_body)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["job_type"], "send_email");
    assert_eq!(created["status"]["state"], "Pending");

    let response = app
        .oneshot(authed_request("GET", &format!("/jobs/{id}"), Body::empty()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let fetched = body_json(response).await;
    assert_eq!(fetched["id"], id);
}

#[tokio::test]
async fn enqueue_rejects_empty_job_type() {
    let app = router();
    let body = json!({"job_type": "", "payload": {}}).to_string();
    let response = app
        .oneshot(authed_request("POST", "/jobs", Body::from(body)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json(response).await;
    assert!(error["error"].as_str().unwrap().contains("job_type"));
}

#[tokio::test]
async fn get_nonexistent_job_returns_404_with_error_envelope() {
    let app = router();
    let missing_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(authed_request(
            "GET",
            &format!("/jobs/{missing_id}"),
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let error = body_json(response).await;
    assert!(error["error"].is_string());
}

#[tokio::test]
async fn list_jobs_filters_by_job_type() {
    let app = router();
    for job_type in ["send_email", "send_email", "generate_report"] {
        let body = json!({"job_type": job_type, "payload": {}}).to_string();
        app.clone()
            .oneshot(authed_request("POST", "/jobs", Body::from(body)))
            .await
            .unwrap();
    }

    let response = app
        .oneshot(authed_request(
            "GET",
            "/jobs?job_type=send_email",
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let jobs = body_json(response).await;
    assert_eq!(jobs.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn cancel_pending_job_succeeds_then_second_cancel_conflicts() {
    let app = router();
    let body = json!({"job_type": "send_email", "payload": {}}).to_string();
    let response = app
        .clone()
        .oneshot(authed_request("POST", "/jobs", Body::from(body)))
        .await
        .unwrap();
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap().to_string();

    let response = app
        .clone()
        .oneshot(authed_request(
            "POST",
            &format!("/jobs/{id}/cancel"),
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Already cancelled (terminal `Failed` state) — cancelling again conflicts.
    let response = app
        .oneshot(authed_request(
            "POST",
            &format!("/jobs/{id}/cancel"),
            Body::empty(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn openapi_json_is_served_without_auth_and_documents_the_job_routes() {
    let response = router()
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let doc = body_json(response).await;
    for path in [
        "/jobs",
        "/jobs/{id}",
        "/jobs/{id}/cancel",
        "/health",
        "/metrics",
    ] {
        assert!(
            doc["paths"][path].is_object(),
            "OpenAPI document is missing path {path}"
        );
    }
    // Schemas referenced from the handlers are auto-collected by utoipa 5.
    assert!(doc["components"]["schemas"]["Job"].is_object());
    assert!(doc["components"]["schemas"]["EnqueueRequest"].is_object());
    // The bearer scheme registered by the SecurityAddon modifier.
    assert!(doc["components"]["securitySchemes"]["bearer_token"].is_object());
}

#[tokio::test]
async fn metrics_endpoint_reports_enqueued_counter() {
    let app = router();
    let body = json!({"job_type": "send_email", "payload": {}}).to_string();
    app.clone()
        .oneshot(authed_request("POST", "/jobs", Body::from(body)))
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.contains("taskforge_jobs_enqueued_total"));
}
