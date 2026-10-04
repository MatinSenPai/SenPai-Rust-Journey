use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use p3_02_04_tower_service_and_layer_middleware::MaintenanceLayer;

/// An app whose one handler counts how many times it actually ran.
fn app(flag: Arc<AtomicBool>, handler_runs: Arc<AtomicUsize>) -> Router {
    Router::new()
        .route(
            "/",
            get(move || async move {
                handler_runs.fetch_add(1, Ordering::SeqCst);
                "hello"
            }),
        )
        .layer(MaintenanceLayer::new(flag))
}

async fn send(app: Router) -> (StatusCode, String) {
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response: Response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn flag_off_passes_the_request_through() {
    let flag = Arc::new(AtomicBool::new(false));
    let runs = Arc::new(AtomicUsize::new(0));

    let (status, body) = send(app(flag, runs.clone())).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "hello");
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn flag_on_answers_503_with_the_maintenance_body() {
    let flag = Arc::new(AtomicBool::new(true));
    let runs = Arc::new(AtomicUsize::new(0));

    let (status, body) = send(app(flag, runs)).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, "down for maintenance");
}

#[tokio::test]
async fn flag_on_never_runs_the_handler() {
    let flag = Arc::new(AtomicBool::new(true));
    let runs = Arc::new(AtomicUsize::new(0));

    send(app(flag, runs.clone())).await;

    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn flipping_the_flag_affects_the_very_next_request() {
    let flag = Arc::new(AtomicBool::new(false));
    let runs = Arc::new(AtomicUsize::new(0));
    let app = app(flag.clone(), runs);

    assert_eq!(send(app.clone()).await.0, StatusCode::OK);
    flag.store(true, Ordering::SeqCst);
    assert_eq!(send(app.clone()).await.0, StatusCode::SERVICE_UNAVAILABLE);
    flag.store(false, Ordering::SeqCst);
    assert_eq!(send(app).await.0, StatusCode::OK);
}
