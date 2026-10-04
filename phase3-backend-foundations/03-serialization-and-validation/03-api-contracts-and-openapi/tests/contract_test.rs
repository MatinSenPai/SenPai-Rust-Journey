//! The four document readers, on the real generated document and on tiny
//! hand-made ones.

use p3_03_03_api_contracts_and_openapi::{
    operation_statuses, response_content_types, schema_names, undocumented, ApiDoc, ROUTES,
};
use serde_json::{json, Value};
use utoipa::OpenApi;

fn real_doc() -> Value {
    serde_json::to_value(ApiDoc::openapi()).unwrap()
}

#[test]
fn schema_names_lists_every_type_the_api_uses_sorted() {
    assert_eq!(
        schema_names(&real_doc()),
        ["Anime", "ApiError", "CreateAnime", "WatchStatus"]
    );
}

#[test]
fn schema_names_sorts_plain_string_order() {
    let doc = json!({"components": {"schemas": {"b": {}, "B": {}, "a": {}}}});
    assert_eq!(schema_names(&doc), ["B", "a", "b"]);
}

#[test]
fn schema_names_of_a_document_without_schemas_is_empty() {
    assert!(schema_names(&json!({"openapi": "3.1.0"})).is_empty());
    assert!(schema_names(&json!({"components": {}})).is_empty());
}

#[test]
fn operation_statuses_reads_the_response_keys_sorted() {
    let doc = real_doc();
    assert_eq!(operation_statuses(&doc, "get", "/anime"), ["200"]);
    assert_eq!(
        operation_statuses(&doc, "get", "/anime/{id}"),
        ["200", "404"]
    );
    assert_eq!(
        operation_statuses(&doc, "delete", "/anime/{id}"),
        ["204", "404"]
    );
    assert_eq!(
        operation_statuses(&doc, "post", "/anime"),
        ["201", "400", "415", "422"]
    );
}

#[test]
fn operation_statuses_ignores_the_case_of_the_method() {
    let doc = real_doc();
    assert_eq!(operation_statuses(&doc, "POST", "/anime").len(), 4);
    assert_eq!(operation_statuses(&doc, "Get", "/anime"), ["200"]);
}

#[test]
fn operation_statuses_of_an_unknown_operation_is_empty() {
    let doc = real_doc();
    assert!(operation_statuses(&doc, "patch", "/anime/{id}").is_empty());
    assert!(operation_statuses(&doc, "get", "/nope").is_empty());
    assert!(operation_statuses(&json!({}), "get", "/anime").is_empty());
}

#[test]
fn response_content_types_tells_json_errors_from_text_errors() {
    let doc = real_doc();
    let ct = |m, p, s| response_content_types(&doc, m, p, s);
    assert_eq!(ct("get", "/anime/{id}", "200"), ["application/json"]);
    assert_eq!(ct("get", "/anime/{id}", "404"), ["application/json"]);
    assert_eq!(ct("post", "/anime", "400"), ["text/plain"]);
    assert_eq!(ct("post", "/anime", "415"), ["text/plain"]);
    assert_eq!(
        ct("POST", "/anime", "422"),
        ["application/json", "text/plain"]
    );
}

#[test]
fn response_content_types_is_empty_when_there_is_no_content() {
    let doc = real_doc();
    let ct = |m, p, s| response_content_types(&doc, m, p, s);
    assert!(ct("delete", "/anime/{id}", "204").is_empty());
    assert!(ct("get", "/anime", "500").is_empty());
    assert!(ct("get", "/nope", "200").is_empty());
}

#[test]
fn undocumented_is_empty_when_the_document_covers_every_route() {
    assert!(undocumented(&real_doc(), ROUTES).is_empty());
}

#[test]
fn undocumented_names_the_missing_routes_in_the_given_order() {
    let routes = [
        ("GET", "/anime"),
        ("PATCH", "/anime/{id}"),
        ("DELETE", "/anime/{id}"),
        ("GET", "/health"),
    ];
    assert_eq!(
        undocumented(&real_doc(), &routes),
        ["PATCH /anime/{id}", "GET /health"]
    );
}

#[test]
fn undocumented_ignores_method_case_and_keeps_the_callers_spelling() {
    let doc = json!({"paths": {"/a": {"get": {}}}});
    let routes = [("get", "/a"), ("Post", "/a"), ("GET", "/b")];
    assert_eq!(undocumented(&doc, &routes), ["Post /a", "GET /b"]);
}

#[test]
fn undocumented_on_an_empty_document_is_every_route() {
    let doc = json!({});
    assert_eq!(undocumented(&doc, &[("GET", "/x")]), ["GET /x"]);
    assert!(undocumented(&doc, &[]).is_empty());
}
