//! Build rung: v1 after its sunset date.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use p3_03_04_api_versioning_and_evolution_solution::{app_after_sunset, AnimeStore};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn get(uri: &str) -> (StatusCode, Option<String>, Value) {
    let router = app_after_sunset(Arc::new(AnimeStore::seeded()));
    let res = router
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let link = res
        .headers()
        .get("link")
        .map(|v| v.to_str().unwrap().to_string());
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (
        status,
        link,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn every_v1_path_is_gone() {
    for uri in ["/v1/anime", "/v1/anime/1", "/v1/anything/at/all"] {
        let (status, link, body) = get(uri).await;
        assert_eq!(status, StatusCode::GONE, "{uri}");
        assert_eq!(
            link.as_deref(),
            Some("</v2/anime>; rel=\"successor-version\"")
        );
        assert_eq!(body, json!({"error":"v1 was retired; use /v2"}));
    }
}

#[tokio::test]
async fn v2_still_works() {
    let (status, _, body) = get("/v2/anime/1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["watch_status"], "watching");
}
