//! One JSON error shape for an entire API — see `README.md` for the theory.
//! `ApiError` is this lesson's centerpiece: every handler below returns
//! `Result<T, ApiError>`, and `impl IntoResponse for ApiError` is the single
//! place that decides what an error *looks like* on the wire. Closely
//! mirrors (but doesn't copy) `capstone-taskforge/taskforge-api/src/error.rs`
//! — read that once you're done here.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// The one JSON shape every error response in this API takes:
/// `{"error": {"code": "not_found", "message": "widget 42 not found"}}`.
/// `code` is machine-readable (a frontend or another service can
/// `match` on it without parsing English out of `message`); `message` is
/// for humans reading logs or an API explorer. Splitting the two is the
/// whole point of this lesson — see `README.md`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

/// Every failure this API can produce, in one enum. `thiserror::Error`
/// derives `Display` (used for logging, via `#[error("...")]` on each
/// variant) and `std::error::Error`, but `Display` alone doesn't tell axum
/// how to turn a value into an HTTP response — that's `IntoResponse`,
/// implemented by hand below.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError::NotFound(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        ApiError::Validation(message.into())
    }

    #[allow(dead_code)]
    pub fn internal(message: impl Into<String>) -> Self {
        ApiError::Internal(message.into())
    }
}

/// The one place in this whole crate that decides "domain error -> HTTP
/// status + JSON body." Every handler below just returns `Result<T,
/// ApiError>` and never touches `StatusCode` or `Json<ErrorBody>` directly —
/// this impl is the only thing standing between them and the wire.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        todo!(
            "match self and destructure each variant's String directly (matching `self` by \
             value moves the String out, no .clone() needed) into a (StatusCode, \
             &'static str, String) tuple of (status, machine-readable code, message): \
             ApiError::NotFound(message) => (StatusCode::NOT_FOUND, \"not_found\", message), \
             ApiError::Validation(message) => (StatusCode::BAD_REQUEST, \"validation_failed\", \
             message), ApiError::Internal(message) => \
             (StatusCode::INTERNAL_SERVER_ERROR, \"internal_error\", message); then return \
             (status, Json(ErrorBody {{ error: ErrorDetail {{ code: code.to_string(), \
             message }} }})).into_response()"
        )
    }
}

/// `validator`'s `.validate()` (see `CreateWidget` below) returns
/// `Result<(), ValidationErrors>` on failure — this `From` impl is what lets
/// a handler write `input.validate()?` inside a function returning
/// `Result<_, ApiError>` and have it just work, the same
/// `From<domain error> for ApiError` pattern `taskforge-api::error` uses for
/// `JobError`.
impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        todo!(
            "build one human-readable String out of `errors`: call errors.field_errors() (a \
             map of field name -> Vec<ValidationError>), for each (field, field_errors) pair \
             and each error in field_errors format!(\"{{field}}: {{msg}}\") where msg is \
             error.message.clone().unwrap_or_else(|| error.code.clone()) — the .message you set \
             via #[validate(..., message = \"...\")] on CreateWidget's fields, falling back to \
             validator's own error code if none was set — collect into a Vec<String>, sort it \
             (HashMap iteration order is unspecified, and sorted output makes tests \
             deterministic), .join(\"; \") into one String, then return \
             ApiError::validation(that_string)"
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Widget {
    pub id: u64,
    pub name: String,
    pub quantity: u32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWidget {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
    #[validate(range(
        min = 1,
        max = 100_000,
        message = "quantity must be between 1 and 100000"
    ))]
    pub quantity: u32,
}

#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Widget>,
}

/// An in-memory catalog: a `HashMap` behind a `Mutex`, gone the moment the
/// process exits — the same shape as `phase3-backend-foundations/02-axum-
/// and-rest-api-design/02-anime-catalog-crud-in-memory`'s `AnimeStore`. What's
/// new in this lesson isn't the store, it's that every fallible method
/// returns `ApiError` directly instead of a separate domain error type, so
/// there's exactly one error type from storage all the way out to the HTTP
/// response.
#[derive(Default)]
pub struct WidgetStore {
    inner: Mutex<StoreInner>,
}

impl WidgetStore {
    pub fn create(&self, input: CreateWidget) -> Result<Widget, ApiError> {
        todo!(
            "call input.validate()? first (the From<ValidationErrors> impl above turns a \
             failure straight into ApiError::Validation); then lock self.inner, increment \
             inner.next_id and use that as the new id, build a Widget from input + the new id, \
             insert a clone into inner.items keyed by id, return Ok(widget)"
        )
    }

    pub fn get(&self, id: u64) -> Result<Widget, ApiError> {
        todo!(
            "lock self.inner, look up id in inner.items, .cloned(), .ok_or_else(|| \
             ApiError::not_found(format!(\"widget {{id}} not found\")))"
        )
    }
}

pub async fn create_widget(
    State(store): State<Arc<WidgetStore>>,
    Json(input): Json<CreateWidget>,
) -> Result<(StatusCode, Json<Widget>), ApiError> {
    todo!(
        "call store.create(input)?; return Ok((StatusCode::CREATED, Json(widget))) — a POST \
         that creates a resource conventionally answers 201, not 200"
    )
}

pub async fn get_widget(
    State(store): State<Arc<WidgetStore>>,
    Path(id): Path<u64>,
) -> Result<Json<Widget>, ApiError> {
    todo!("store.get(id).map(Json)")
}

/// `/widgets` handles creating; `/widgets/{id}` handles fetching one.
pub fn app(store: Arc<WidgetStore>) -> Router {
    todo!(
        "Router::new() \
            .route(\"/widgets\", post(create_widget)) \
            .route(\"/widgets/{{id}}\", get(get_widget)) \
            .with_state(store)"
    )
}
