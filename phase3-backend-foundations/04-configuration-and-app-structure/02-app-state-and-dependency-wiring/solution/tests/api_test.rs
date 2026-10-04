//! The whole stack (router, `FromRef` sub-states, handlers, store) driven with
//! `oneshot` and a fake clock: real requests, no socket, no sleeping.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_04_02_app_state_and_dependency_wiring_solution::{app, AppState, Config, FakeClock};

/// A fresh app on a fake clock reading `start`. The test keeps the clock so it
/// can move time; the app holds another handle to the same clock.
fn fixture(start: u64, config: Config) -> (Router, Arc<FakeClock>) {
    let clock = Arc::new(FakeClock::at(start));
    let state = AppState::new(clock.clone(), config);
    (app(state), clock)
}

async fn send(
    router: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    };
    let response = router.clone().oneshot(request.unwrap()).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn titles(body: &str) -> Vec<String> {
    let list: Vec<Value> = serde_json::from_str(body).unwrap();
    list.iter()
        .map(|w| w["title"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn posting_a_watch_answers_201_and_stamps_it_with_the_clock() {
    let (router, _) = fixture(1_700_000_000, Config::default());
    let (status, body) = send(&router, "POST", "/watch", Some(json!({"title": "Frieren"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    let value: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        value,
        json!({"id": 1, "title": "Frieren", "watched_at": 1_700_000_000u64})
    );
}

#[tokio::test]
async fn the_stamp_follows_the_fake_clock() {
    let (router, clock) = fixture(1000, Config::default());
    send(&router, "POST", "/watch", Some(json!({"title": "a"}))).await;
    clock.advance(250);
    let (_, body) = send(&router, "POST", "/watch", Some(json!({"title": "b"}))).await;
    let value: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(value["watched_at"], 1250);
    assert_eq!(value["id"], 2);
}

#[tokio::test]
async fn the_title_is_trimmed_before_it_is_stored() {
    let (router, _) = fixture(0, Config::default());
    let (_, body) = send(
        &router,
        "POST",
        "/watch",
        Some(json!({"title": "  Frieren \n"})),
    )
    .await;
    let value: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(value["title"], "Frieren");
}

#[tokio::test]
async fn a_blank_title_is_422_and_records_nothing() {
    let (router, _) = fixture(0, Config::default());
    for blank in ["", "   ", "\t\n"] {
        let (status, body) = send(&router, "POST", "/watch", Some(json!({"title": blank}))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{blank:?}");
        assert_eq!(body, "invalid title", "{blank:?}");
    }
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn the_title_limit_comes_from_the_config_in_the_state() {
    let config = Config {
        max_title_len: 5,
        ..Config::default()
    };
    let (router, _) = fixture(0, config);
    let (status, _) = send(&router, "POST", "/watch", Some(json!({"title": "12345"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = send(&router, "POST", "/watch", Some(json!({"title": "123456"}))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, "invalid title");
}

#[tokio::test]
async fn the_title_limit_counts_characters_not_bytes() {
    let config = Config {
        max_title_len: 5,
        ..Config::default()
    };
    let (router, _) = fixture(0, config);
    // five characters, more than five bytes
    let (status, _) = send(&router, "POST", "/watch", Some(json!({"title": "é é é"}))).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn recent_lists_only_what_fits_in_the_window() {
    let config = Config {
        recent_window_secs: 100,
        ..Config::default()
    };
    let (router, clock) = fixture(1000, config);
    send(&router, "POST", "/watch", Some(json!({"title": "first"}))).await;
    clock.advance(60);
    send(&router, "POST", "/watch", Some(json!({"title": "second"}))).await;
    clock.advance(30);
    // now = 1090: first is 90s old, second is 30s old
    let (status, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(titles(&body), ["first", "second"]);
    clock.advance(20);
    // now = 1110: first is 110s old and has aged out
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(titles(&body), ["second"]);
    clock.advance(1_000);
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn an_entry_exactly_one_window_old_still_counts() {
    let config = Config {
        recent_window_secs: 100,
        ..Config::default()
    };
    let (router, clock) = fixture(500, config);
    send(&router, "POST", "/watch", Some(json!({"title": "edge"}))).await;
    clock.advance(100);
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(titles(&body), ["edge"]);
    clock.advance(1);
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn a_clock_smaller_than_the_window_does_not_underflow() {
    let (router, _) = fixture(10, Config::default()); // window is 3600
    send(&router, "POST", "/watch", Some(json!({"title": "early"}))).await;
    let (status, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(titles(&body), ["early"]);
}

#[tokio::test]
async fn every_request_sees_the_same_store() {
    let (router, _) = fixture(0, Config::default());
    for title in ["a", "b", "c"] {
        send(&router, "POST", "/watch", Some(json!({"title": title}))).await;
    }
    let (_, body) = send(&router, "GET", "/watch/recent", None).await;
    assert_eq!(titles(&body), ["a", "b", "c"]);
}

#[tokio::test]
async fn two_apps_do_not_share_a_store() {
    let (one, _) = fixture(0, Config::default());
    let (two, _) = fixture(0, Config::default());
    send(
        &one,
        "POST",
        "/watch",
        Some(json!({"title": "only in one"})),
    )
    .await;
    let (_, body) = send(&two, "GET", "/watch/recent", None).await;
    assert_eq!(body, "[]");
}

#[tokio::test]
async fn an_unknown_method_is_405() {
    let (router, _) = fixture(0, Config::default());
    let (status, _) = send(&router, "DELETE", "/watch", None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}
