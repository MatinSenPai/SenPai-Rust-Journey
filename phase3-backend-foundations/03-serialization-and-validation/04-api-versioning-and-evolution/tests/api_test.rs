//! The router, driven through `tower::ServiceExt::oneshot`: real requests, no socket.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use p3_03_04_api_versioning_and_evolution::{app, AnimeStore};
use serde_json::{json, Value};
use tower::ServiceExt;

fn router() -> Router {
    app(Arc::new(AnimeStore::seeded()))
}

async fn get(uri: &str, accept: Option<&str>) -> (StatusCode, HeaderMap, Value) {
    let mut req = Request::get(uri);
    if let Some(a) = accept {
        req = req.header("accept", a);
    }
    let res = router()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let (status, headers) = (res.status(), res.headers().clone());
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (
        status,
        headers,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn v1_answers_the_frozen_shape() {
    let (status, _, body) = get("/v1/anime/1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"id":1,"title":"Frieren","status":"watching","rating":9})
    );
}

#[tokio::test]
async fn v2_answers_the_new_shape() {
    let (status, _, body) = get("/v2/anime/1", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"id":1,"title":"Frieren","watch_status":"watching","rating":9,"episodes":28})
    );
}

#[tokio::test]
async fn the_lists_use_the_same_shapes() {
    let (_, _, v1) = get("/v1/anime", None).await;
    let (_, _, v2) = get("/v2/anime", None).await;
    assert_eq!(
        v1[1],
        json!({"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null})
    );
    assert_eq!(
        v2[1],
        json!({"id":2,"title":"Dandadan","watch_status":"plan_to_watch","rating":null})
    );
}

#[tokio::test]
async fn v1_responses_carry_the_lifecycle_headers_even_on_404() {
    for uri in ["/v1/anime", "/v1/anime/1", "/v1/anime/99"] {
        let (_, h, _) = get(uri, None).await;
        assert_eq!(h["deprecation"], "@1767225600", "{uri}");
        assert_eq!(h["sunset"], "Thu, 31 Dec 2026 23:59:59 GMT", "{uri}");
        assert_eq!(h["link"], "</v2/anime>; rel=\"successor-version\"", "{uri}");
    }
}

#[tokio::test]
async fn v2_has_no_lifecycle_headers() {
    let (_, h, _) = get("/v2/anime/1", None).await;
    assert!(h.get("deprecation").is_none());
    assert!(h.get("sunset").is_none());
}

#[tokio::test]
async fn a_missing_id_is_404_with_the_error_body_on_every_route() {
    for uri in ["/v1/anime/99", "/v2/anime/99", "/anime/99"] {
        let (status, _, body) = get(uri, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body, json!({"error":"anime not found"}), "{uri}");
    }
}

#[tokio::test]
async fn the_unversioned_route_follows_accept_and_says_vary() {
    let (_, h, v1) = get("/anime/1", None).await;
    assert_eq!(v1["status"], "watching");
    assert_eq!(h["vary"], "accept");
    let (_, h, v2) = get("/anime/1", Some("application/vnd.anime.v2+json")).await;
    assert_eq!(v2["watch_status"], "watching");
    assert_eq!(h["vary"], "accept");
    assert!(h.get("deprecation").is_none());
}
