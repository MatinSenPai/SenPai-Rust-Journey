//! The error layer on its own: plain function calls, no router, no requests.

use axum::body::to_bytes;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use serde_json::{json, Value};

use p3_08_01_consistent_error_envelopes_solution::{
    validate_new_show, ApiError, FieldError, NewShow,
};

fn all_variants() -> Vec<ApiError> {
    vec![
        ApiError::NotFound("x".into()),
        ApiError::MethodNotAllowed,
        ApiError::BadRequest("x".into()),
        ApiError::UnsupportedMediaType("x".into()),
        ApiError::InvalidBody("x".into()),
        ApiError::Validation(vec![]),
        ApiError::Internal("x".into()),
    ]
}

async fn body_of(error: ApiError) -> (StatusCode, String, Value) {
    let response = error.into_response();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        content_type,
        serde_json::from_slice(&bytes).unwrap(),
    )
}

#[test]
fn status_of_every_variant() {
    let statuses: Vec<u16> = all_variants().iter().map(|e| e.status().as_u16()).collect();
    assert_eq!(statuses, [404, 405, 400, 415, 422, 422, 500]);
}

#[test]
fn code_of_every_variant() {
    let codes: Vec<&str> = all_variants().iter().map(ApiError::code).collect();
    assert_eq!(
        codes,
        [
            "not_found",
            "method_not_allowed",
            "bad_request",
            "unsupported_media_type",
            "invalid_body",
            "validation_failed",
            "internal_error",
        ]
    );
}

#[test]
fn message_passes_client_safe_text_through() {
    assert_eq!(
        ApiError::NotFound("show 7 not found".into()).message(),
        "show 7 not found"
    );
    assert_eq!(ApiError::BadRequest("bad id".into()).message(), "bad id");
    assert_eq!(
        ApiError::InvalidBody("no title".into()).message(),
        "no title"
    );
    assert_eq!(
        ApiError::UnsupportedMediaType("send json".into()).message(),
        "send json"
    );
}

#[test]
fn message_for_the_variants_without_text() {
    assert_eq!(
        ApiError::MethodNotAllowed.message(),
        "method not allowed for this route"
    );
    assert_eq!(
        ApiError::Validation(vec![]).message(),
        "the request body has invalid fields"
    );
}

#[test]
fn an_internal_message_never_contains_the_inner_text() {
    let message = ApiError::Internal("password=hunter2".into()).message();
    assert_eq!(message, "something went wrong on our side");
}

#[tokio::test]
async fn into_response_builds_the_envelope() {
    let (status, content_type, body) = body_of(ApiError::NotFound("show 7 not found".into())).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(content_type, "application/json");
    assert_eq!(
        body,
        json!({"error": {"code": "not_found", "message": "show 7 not found"}})
    );
}

#[tokio::test]
async fn into_response_includes_fields_only_for_validation() {
    let fields = vec![FieldError {
        field: "title",
        code: "length",
        message: "title must be 1 to 100 characters".into(),
    }];
    let (status, _, body) = body_of(ApiError::Validation(fields)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body,
        json!({"error": {
            "code": "validation_failed",
            "message": "the request body has invalid fields",
            "fields": [{
                "field": "title",
                "code": "length",
                "message": "title must be 1 to 100 characters"
            }]
        }})
    );
}

#[tokio::test]
async fn into_response_hides_the_internal_detail() {
    let (status, _, body) = body_of(ApiError::Internal("password=hunter2".into())).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!body.to_string().contains("hunter2"));
    assert_eq!(body["error"]["code"], "internal_error");
}

fn show(title: &str, episodes: u32) -> NewShow {
    NewShow {
        title: title.to_string(),
        episodes,
    }
}

fn fields_of(result: Result<(), ApiError>) -> Vec<FieldError> {
    match result {
        Err(ApiError::Validation(fields)) => fields,
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn a_valid_show_passes() {
    assert!(validate_new_show(&show("Frieren", 28)).is_ok());
}

#[test]
fn the_limits_are_inclusive() {
    assert!(validate_new_show(&show("a", 1)).is_ok());
    assert!(validate_new_show(&show(&"a".repeat(100), 2000)).is_ok());
}

#[test]
fn an_empty_title_is_reported_with_its_code_and_message() {
    let fields = fields_of(validate_new_show(&show("", 5)));
    assert_eq!(
        fields,
        [FieldError {
            field: "title",
            code: "length",
            message: "title must be 1 to 100 characters".into(),
        }]
    );
}

#[test]
fn an_out_of_range_episode_count_is_reported_with_its_code_and_message() {
    for episodes in [0, 2001] {
        let fields = fields_of(validate_new_show(&show("Frieren", episodes)));
        assert_eq!(
            fields,
            [FieldError {
                field: "episodes",
                code: "range",
                message: "episodes must be 1 to 2000".into(),
            }]
        );
    }
}

#[test]
fn every_broken_rule_is_reported_title_first() {
    let fields = fields_of(validate_new_show(&show(&"a".repeat(101), 0)));
    let names: Vec<&str> = fields.iter().map(|f| f.field).collect();
    assert_eq!(names, ["title", "episodes"]);
}

#[test]
fn title_length_counts_characters_not_bytes() {
    // 100 Persian letters are 200 bytes in UTF-8 but 100 characters.
    assert!(validate_new_show(&show(&"ا".repeat(100), 1)).is_ok());
    assert!(validate_new_show(&show(&"ا".repeat(101), 1)).is_err());
}
