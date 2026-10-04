//! The whole stack through `tower::ServiceExt::oneshot`. The point of the
//! lesson is that *every* failure has the same shape, so most tests check
//! status and envelope together.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_08_01_consistent_error_envelopes::{app, ShowStore};

struct Reply {
    status: StatusCode,
    content_type: String,
    body: Vec<u8>,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
    fn error(&self) -> Value {
        self.json()["error"].clone()
    }
}

async fn send(store: &Arc<ShowStore>, request: Request<Body>) -> Reply {
    let response = app(store.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        content_type,
        body: body.to_vec(),
    }
}

fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).body(Body::empty()).unwrap()
}

fn post_json(raw: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/shows")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(raw.to_string()))
        .unwrap()
}

fn fresh() -> Arc<ShowStore> {
    Arc::new(ShowStore::default())
}

#[tokio::test]
async fn create_then_get() {
    let store = fresh();
    let created = send(&store, post_json(r#"{"title":"Frieren","episodes":28}"#)).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(
        created.json(),
        json!({"id": 1, "title": "Frieren", "episodes": 28})
    );
    let fetched = send(&store, get("/shows/1")).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.json(), created.json());
}

#[tokio::test]
async fn a_missing_show_is_a_404_envelope() {
    let reply = send(&fresh(), get("/shows/9999")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.content_type, "application/json");
    assert_eq!(
        reply.error(),
        json!({"code": "not_found", "message": "show 9999 not found"})
    );
}

#[tokio::test]
async fn broken_rules_are_a_422_with_one_entry_per_field() {
    let reply = send(&fresh(), post_json(r#"{"title":"","episodes":0}"#)).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    let error = reply.error();
    assert_eq!(error["code"], "validation_failed");
    assert_eq!(error["message"], "the request body has invalid fields");
    assert_eq!(
        error["fields"],
        json!([
            {"field": "title", "code": "length", "message": "title must be 1 to 100 characters"},
            {"field": "episodes", "code": "range", "message": "episodes must be 1 to 2000"}
        ])
    );
}

#[tokio::test]
async fn a_rejected_create_does_not_store_anything() {
    let store = fresh();
    send(&store, post_json(r#"{"title":"","episodes":1}"#)).await;
    let reply = send(&store, get("/shows/1")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn broken_json_is_a_400_envelope_not_plain_text() {
    let reply = send(&fresh(), post_json("{")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.content_type, "application/json");
    assert_eq!(reply.error()["code"], "bad_request");
    assert!(!reply.error()["message"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn json_of_the_wrong_shape_is_a_422_invalid_body() {
    for raw in [r#"{"episodes":3}"#, r#"{"title":"x","episodes":-1}"#] {
        let reply = send(&fresh(), post_json(raw)).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{raw}");
        assert_eq!(reply.error()["code"], "invalid_body", "{raw}");
        assert!(reply.error().get("fields").is_none(), "{raw}");
    }
}

#[tokio::test]
async fn a_missing_content_type_is_a_415_envelope() {
    let request = Request::builder()
        .method("POST")
        .uri("/shows")
        .body(Body::from(r#"{"title":"x","episodes":1}"#))
        .unwrap();
    let reply = send(&fresh(), request).await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(reply.error()["code"], "unsupported_media_type");
}

#[tokio::test]
async fn an_id_that_is_not_a_number_is_a_400_envelope() {
    let reply = send(&fresh(), get("/shows/abc")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.content_type, "application/json");
    assert_eq!(reply.error()["code"], "bad_request");
}

#[tokio::test]
async fn an_unknown_route_is_a_404_envelope() {
    let reply = send(&fresh(), get("/nope/at/all")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.content_type, "application/json");
    assert_eq!(
        reply.error(),
        json!({"code": "not_found", "message": "no route for /nope/at/all"})
    );
}

#[tokio::test]
async fn a_wrong_method_is_a_405_envelope() {
    let request = Request::builder()
        .method("DELETE")
        .uri("/shows/1")
        .body(Body::empty())
        .unwrap();
    let reply = send(&fresh(), request).await;
    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(reply.content_type, "application/json");
    assert_eq!(reply.error()["code"], "method_not_allowed");
}

#[tokio::test]
async fn a_server_failure_is_a_500_that_leaks_nothing() {
    let reply = send(&fresh(), get("/simulate-failure")).await;
    assert_eq!(reply.status, StatusCode::INTERNAL_SERVER_ERROR);
    let text = String::from_utf8(reply.body.clone()).unwrap();
    assert!(!text.contains("hunter2"), "{text}");
    assert!(!text.contains("postgres"), "{text}");
    assert_eq!(
        reply.error(),
        json!({"code": "internal_error", "message": "something went wrong on our side"})
    );
}

#[tokio::test]
async fn every_error_has_exactly_one_top_level_key_and_a_code_and_message() {
    let store = fresh();
    let requests = vec![
        get("/shows/1"),
        get("/shows/abc"),
        get("/nope"),
        get("/simulate-failure"),
        post_json("{"),
        post_json(r#"{"title":"","episodes":1}"#),
    ];
    for request in requests {
        let uri = request.uri().to_string();
        let reply = send(&store, request).await;
        assert!(reply.status.is_client_error() || reply.status.is_server_error());
        let body = reply.json();
        let top: Vec<&String> = body.as_object().unwrap().keys().collect();
        assert_eq!(top, ["error"], "{uri}");
        assert!(body["error"]["code"].is_string(), "{uri}");
        assert!(body["error"]["message"].is_string(), "{uri}");
    }
}
