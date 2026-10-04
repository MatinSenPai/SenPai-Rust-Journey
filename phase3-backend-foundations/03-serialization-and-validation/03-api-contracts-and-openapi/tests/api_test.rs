//! The contract and the behaviour, side by side: the document is served, and
//! every status it promises for `/anime` is one the router really answers.
//! No socket: requests go through `tower::ServiceExt::oneshot`.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_03_03_api_contracts_and_openapi::{app, Catalog};

struct Reply {
    status: StatusCode,
    content_type: Option<String>,
    body: Vec<u8>,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
}

async fn send(
    catalog: &Arc<Catalog>,
    method: &str,
    uri: &str,
    body: Option<(&str, &str)>,
) -> Reply {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some((content_type, text)) => builder
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(text.to_string())),
        None => builder.body(Body::empty()),
    };
    let response = app(catalog.clone())
        .oneshot(request.unwrap())
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string());
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        content_type,
        body: body.to_vec(),
    }
}

fn post_json(text: &str) -> Option<(&str, &str)> {
    Some(("application/json", text))
}

const FRIEREN: &str = r#"{"title":"Frieren","status":"watching","rating":9}"#;

#[tokio::test]
async fn the_document_is_served_as_openapi_3_1_json() {
    let catalog = Arc::new(Catalog::default());
    let reply = send(&catalog, "GET", "/api-docs/openapi.json", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.content_type.as_deref(), Some("application/json"));
    let doc = reply.json();
    assert_eq!(doc["openapi"], "3.1.0");
    assert_eq!(doc["info"]["title"], "Anime catalog");
    assert_eq!(doc["info"]["version"], "1.0.0");
}

#[tokio::test]
async fn the_served_document_has_the_four_operations() {
    let catalog = Arc::new(Catalog::default());
    let doc = send(&catalog, "GET", "/api-docs/openapi.json", None)
        .await
        .json();
    assert!(doc["paths"]["/anime"]["get"].is_object());
    assert!(doc["paths"]["/anime"]["post"].is_object());
    assert!(doc["paths"]["/anime/{id}"]["get"].is_object());
    assert!(doc["paths"]["/anime/{id}"]["delete"].is_object());
}

#[tokio::test]
async fn created_answers_201_with_the_new_anime() {
    let catalog = Arc::new(Catalog::default());
    let reply = send(&catalog, "POST", "/anime", post_json(FRIEREN)).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    assert_eq!(
        reply.json(),
        json!({"id":1,"title":"Frieren","status":"watching","rating":9})
    );
}

#[tokio::test]
async fn a_rating_of_15_is_a_422_with_the_json_error_shape() {
    let catalog = Arc::new(Catalog::default());
    let body = r#"{"title":"Bad","status":"watching","rating":15}"#;
    let reply = send(&catalog, "POST", "/anime", post_json(body)).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.content_type.as_deref(), Some("application/json"));
    assert_eq!(
        reply.json(),
        json!({"error": "rating must be between 1 and 10, got 15"})
    );
}

#[tokio::test]
async fn axum_rejections_are_plain_text_as_the_contract_says() {
    let catalog = Arc::new(Catalog::default());

    let broken = send(&catalog, "POST", "/anime", post_json("{")).await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    assert!(broken.content_type.unwrap().starts_with("text/plain"));

    let no_ct = send(&catalog, "POST", "/anime", Some(("text/plain", FRIEREN))).await;
    assert_eq!(no_ct.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(no_ct.content_type.unwrap().starts_with("text/plain"));

    let shape = post_json(r#"{"status":"watching"}"#);
    let wrong_shape = send(&catalog, "POST", "/anime", shape).await;
    assert_eq!(wrong_shape.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(wrong_shape.content_type.unwrap().starts_with("text/plain"));
}

#[tokio::test]
async fn get_and_delete_follow_the_documented_statuses() {
    let catalog = Arc::new(Catalog::default());
    send(&catalog, "POST", "/anime", post_json(FRIEREN)).await;

    let found = send(&catalog, "GET", "/anime/1", None).await;
    assert_eq!(found.status, StatusCode::OK);

    let missing = send(&catalog, "GET", "/anime/99", None).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.json(), json!({"error": "anime not found"}));

    let deleted = send(&catalog, "DELETE", "/anime/1", None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert!(deleted.body.is_empty());

    let again = send(&catalog, "DELETE", "/anime/1", None).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_is_a_json_array_in_creation_order() {
    let catalog = Arc::new(Catalog::default());
    send(&catalog, "POST", "/anime", post_json(FRIEREN)).await;
    let second = r#"{"title":"Dandadan","status":"plan_to_watch"}"#;
    send(&catalog, "POST", "/anime", post_json(second)).await;
    let list = send(&catalog, "GET", "/anime", None).await.json();
    let titles: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["Frieren", "Dandadan"]);
}
