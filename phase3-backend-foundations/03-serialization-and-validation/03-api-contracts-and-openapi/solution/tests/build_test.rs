//! The Build rung, black box: whatever you named your handler, `PUT
//! /anime/{id}` must work and the document must say so.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use utoipa::OpenApi;

use p3_03_03_api_contracts_and_openapi_solution::{
    app, operation_statuses, undocumented, ApiDoc, Catalog, ROUTES,
};

async fn send(
    catalog: &Arc<Catalog>,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    };
    let response = app(catalog.clone())
        .oneshot(request.unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn put_replaces_the_whole_anime_and_keeps_the_id() {
    let catalog = Arc::new(Catalog::default());
    let first = json!({"title":"Frieren","status":"watching","rating":9});
    send(&catalog, "POST", "/anime", Some(first)).await;
    let new = json!({"title":"Frieren S2","status":"plan_to_watch"});
    let reply = send(&catalog, "PUT", "/anime/1", Some(new.clone())).await;
    assert_eq!(reply.0, StatusCode::OK);
    assert_eq!(
        reply.1,
        json!({"id":1,"title":"Frieren S2","status":"plan_to_watch","rating":null})
    );
    let again = send(&catalog, "PUT", "/anime/1", Some(new)).await;
    assert_eq!(reply, again);
}

#[tokio::test]
async fn put_on_a_missing_id_is_404_and_a_bad_rating_is_422() {
    let catalog = Arc::new(Catalog::default());
    let ok = json!({"title":"X","status":"watching"});
    let (status, _) = send(&catalog, "PUT", "/anime/7", Some(ok.clone())).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    send(&catalog, "POST", "/anime", Some(ok)).await;
    let bad = json!({"title":"X","status":"watching","rating":11});
    let (status, _) = send(&catalog, "PUT", "/anime/1", Some(bad)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[test]
fn the_document_describes_put_and_routes_lists_it() {
    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let statuses = operation_statuses(&doc, "put", "/anime/{id}");
    for code in ["200", "404", "422"] {
        assert!(statuses.contains(&code.to_string()), "missing {code}");
    }
    assert!(ROUTES.contains(&("PUT", "/anime/{id}")));
    assert!(undocumented(&doc, ROUTES).is_empty());
}
