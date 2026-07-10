use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use sq_03_webtoon_notifier_service_solution::{
    app, AlwaysNewChapterChecker, AppState, FollowWebtoon, NeverNewChapterChecker, WebtoonStore,
};

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn fresh_app_always_new() -> axum::Router {
    app(AppState {
        store: Arc::new(WebtoonStore::default()),
        checker: Arc::new(AlwaysNewChapterChecker),
    })
}

#[tokio::test]
async fn follow_then_list_returns_the_followed_webtoon() {
    let state = AppState {
        store: Arc::new(WebtoonStore::default()),
        checker: Arc::new(NeverNewChapterChecker),
    };

    let response = app(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webtoons")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"title": "Solo Leveling", "current_chapter": 179}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    assert_eq!(created["title"], "Solo Leveling");
    assert_eq!(created["current_chapter"], 179);

    let response = app(state)
        .oneshot(
            Request::builder()
                .uri("/webtoons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = json_body(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_starts_empty() {
    let response = fresh_app_always_new()
        .oneshot(
            Request::builder()
                .uri("/webtoons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await, json!([]));
}

#[tokio::test]
async fn checking_a_webtoon_with_a_new_chapter_updates_it_and_reports_true() {
    let store = Arc::new(WebtoonStore::default());
    let webtoon = store.follow(FollowWebtoon {
        title: "Tower of God".to_string(),
        current_chapter: 600,
    });
    let state = AppState {
        store: store.clone(),
        checker: Arc::new(AlwaysNewChapterChecker),
    };

    let response = app(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/webtoons/{}/check", webtoon.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let result = json_body(response).await;
    assert_eq!(result["new_chapter_found"], true);
    assert_eq!(result["webtoon"]["current_chapter"], 601);
    assert_eq!(store.get(webtoon.id).unwrap().current_chapter, 601);
}

#[tokio::test]
async fn checking_a_webtoon_with_no_new_chapter_reports_false_and_leaves_it_unchanged() {
    let store = Arc::new(WebtoonStore::default());
    let webtoon = store.follow(FollowWebtoon {
        title: "Omniscient Reader".to_string(),
        current_chapter: 42,
    });
    let state = AppState {
        store: store.clone(),
        checker: Arc::new(NeverNewChapterChecker),
    };

    let response = app(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/webtoons/{}/check", webtoon.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let result = json_body(response).await;
    assert_eq!(result["new_chapter_found"], false);
    assert_eq!(result["webtoon"]["current_chapter"], 42);
}

#[tokio::test]
async fn checking_an_unknown_webtoon_id_returns_404() {
    let response = fresh_app_always_new()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webtoons/9999/check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let error = json_body(response).await;
    assert_eq!(error["error"], "webtoon not found");
}
