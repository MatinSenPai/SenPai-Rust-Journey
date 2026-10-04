//! Solution for 3.8.1 — Consistent error envelopes.
//!
//! One enum (`ApiError`), one `IntoResponse` impl, and every failure in the
//! API, including the ones `axum` produces before a handler runs, leaves
//! through it as `{"error": {"code", "message", "fields"?}}`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRequest, FromRequestParts, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// The wire shape of every error: `{"error": {...}}`.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// The inside of the envelope. `fields` is left out of the JSON entirely
/// when it is empty, so only validation failures carry it.
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldError>,
}

/// One broken rule on one field of a request body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: &'static str,
    pub code: &'static str,
    pub message: String,
}

/// Every way a request to this API can fail.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unsupported media type: {0}")]
    UnsupportedMediaType(String),
    #[error("invalid body: {0}")]
    InvalidBody(String),
    #[error("validation failed: {0:?}")]
    Validation(Vec<FieldError>),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// The HTTP status of this failure.
    ///
    /// | Variant | Status |
    /// |---|---|
    /// | `NotFound` | `404` |
    /// | `MethodNotAllowed` | `405` |
    /// | `BadRequest` | `400` |
    /// | `Conflict` | `409` |
    /// | `UnsupportedMediaType` | `415` |
    /// | `InvalidBody` | `422` |
    /// | `Validation` | `422` |
    /// | `Internal` | `500` |
    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::InvalidBody(_) | ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stable, machine-readable code of this failure. One per variant:
    /// `not_found`, `method_not_allowed`, `bad_request`,
    /// `conflict`, `unsupported_media_type`, `invalid_body`, `validation_failed` (for
    /// `Validation`) and `internal_error` (for `Internal`).
    pub fn code(&self) -> &'static str {
        match self {
            ApiError::NotFound(_) => "not_found",
            ApiError::MethodNotAllowed => "method_not_allowed",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Conflict(_) => "conflict",
            ApiError::UnsupportedMediaType(_) => "unsupported_media_type",
            ApiError::InvalidBody(_) => "invalid_body",
            ApiError::Validation(_) => "validation_failed",
            ApiError::Internal(_) => "internal_error",
        }
    }

    /// The human-readable message that is safe to show a client.
    ///
    /// - `NotFound`, `BadRequest`, `Conflict`, `UnsupportedMediaType` and `InvalidBody`:
    ///   the text they carry, unchanged.
    /// - `MethodNotAllowed`: exactly `"method not allowed for this route"`.
    /// - `Validation`: exactly `"the request body has invalid fields"`.
    /// - `Internal`: exactly `"something went wrong on our side"`. The text
    ///   inside `Internal` is for the server's own log and must never appear
    ///   here.
    pub fn message(&self) -> String {
        match self {
            ApiError::NotFound(m)
            | ApiError::BadRequest(m)
            | ApiError::Conflict(m)
            | ApiError::UnsupportedMediaType(m)
            | ApiError::InvalidBody(m) => m.clone(),
            ApiError::MethodNotAllowed => "method not allowed for this route".to_string(),
            ApiError::Validation(_) => "the request body has invalid fields".to_string(),
            ApiError::Internal(_) => "something went wrong on our side".to_string(),
        }
    }
}

impl ApiError {
    /// The same failure as an RFC 9457 problem: the status of [`ApiError::status`],
    /// `Content-Type: application/problem+json`, and a body with `type`
    /// (`https://api.example.com/problems/<code>`), `title` (the status's
    /// canonical reason phrase), `status`, `detail` (the text of
    /// [`ApiError::message`]), `instance`, and, for `Validation` only, an
    /// `errors` array of the field entries.
    pub fn problem_response(&self, instance: &str) -> Response {
        let status = self.status();
        let mut body = serde_json::json!({
            "type": format!("https://api.example.com/problems/{}", self.code()),
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": self.message(),
            "instance": instance,
        });
        if let ApiError::Validation(fields) = self {
            body["errors"] = serde_json::json!(fields);
        }
        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            body.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for ApiError {
    /// Builds the response: [`ApiError::status`] as the status code, and a
    /// JSON body (`Content-Type: application/json`) of the shape described on [`ErrorBody`] with
    /// [`ApiError::code`] and [`ApiError::message`]. `fields` holds the
    /// `FieldError`s of a `Validation` and is empty (so absent from the JSON)
    /// for every other variant. An `Internal` error also prints its inner
    /// text to standard error as `internal error: <text>`, because the
    /// server's operator needs it even though the client does not get it.
    fn into_response(self) -> Response {
        if let ApiError::Internal(detail) = &self {
            eprintln!("internal error: {detail}");
        }
        let status = self.status();
        let code = self.code();
        let message = self.message();
        let fields = match self {
            ApiError::Validation(fields) => fields,
            _ => Vec::new(),
        };
        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message,
                fields,
            },
        };
        (status, Json(body)).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    /// Turns `axum`'s `Json` rejection into an `ApiError` whose message is the
    /// rejection's own text (`body_text()`). A rejection with status `415`
    /// becomes `UnsupportedMediaType`, one with status `422` becomes
    /// `InvalidBody`, and every other status becomes `BadRequest`.
    fn from(rejection: JsonRejection) -> Self {
        let message = rejection.body_text();
        match rejection.status() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::UnsupportedMediaType(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::InvalidBody(message),
            _ => ApiError::BadRequest(message),
        }
    }
}

impl From<PathRejection> for ApiError {
    /// Turns `axum`'s `Path` rejection into an `ApiError` carrying the
    /// rejection's own text (`body_text()`). A rejection with status `500`
    /// (the router itself is misconfigured, which is never the client's
    /// fault) becomes `Internal`; every other status becomes `BadRequest`.
    fn from(rejection: PathRejection) -> Self {
        let message = rejection.body_text();
        if rejection.status() == StatusCode::INTERNAL_SERVER_ERROR {
            ApiError::Internal(message)
        } else {
            ApiError::BadRequest(message)
        }
    }
}

/// `Json<T>`, except that its rejection is an [`ApiError`].
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

/// `Path<T>`, except that its rejection is an [`ApiError`].
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);

/// One show in the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Show {
    pub id: u64,
    pub title: String,
    pub episodes: u32,
}

/// The body of `POST /shows`.
#[derive(Debug, Deserialize)]
pub struct NewShow {
    pub title: String,
    pub episodes: u32,
}

/// Checks the business rules of a [`NewShow`] and reports every broken one.
///
/// - `title` must have `1..=100` characters (counted as `char`s, exactly as
///   sent, no trimming). Otherwise: field `"title"`, code `"length"`,
///   message `"title must be 1 to 100 characters"`.
/// - `episodes` must be `1..=2000`. Otherwise: field `"episodes"`, code
///   `"range"`, message `"episodes must be 1 to 2000"`.
///
/// Returns `Ok(())` when both hold. Otherwise returns
/// `ApiError::Validation` with every broken rule, the `title` one first.
pub fn validate_new_show(input: &NewShow) -> Result<(), ApiError> {
    let mut fields = Vec::new();
    if !(1..=100).contains(&input.title.chars().count()) {
        fields.push(FieldError {
            field: "title",
            code: "length",
            message: "title must be 1 to 100 characters".to_string(),
        });
    }
    if !(1..=2000).contains(&input.episodes) {
        fields.push(FieldError {
            field: "episodes",
            code: "range",
            message: "episodes must be 1 to 2000".to_string(),
        });
    }
    if fields.is_empty() {
        Ok(())
    } else {
        Err(ApiError::Validation(fields))
    }
}

#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Show>,
}

/// An in-memory catalog. Plain Rust: no `axum`, no status codes.
#[derive(Default)]
pub struct ShowStore {
    inner: Mutex<StoreInner>,
}

impl ShowStore {
    /// Stores the show under a fresh id (the first is `1`) and returns it, or
    /// `Conflict` (message `a show titled "<title>" already exists`) when a
    /// stored show has exactly this title. Check and insert happen under one lock.
    pub fn insert(&self, input: NewShow) -> Result<Show, ApiError> {
        let mut inner = self.inner.lock().unwrap();
        if inner.items.values().any(|s| s.title == input.title) {
            return Err(ApiError::Conflict(format!(
                "a show titled \"{}\" already exists",
                input.title
            )));
        }
        inner.next_id += 1;
        let show = Show {
            id: inner.next_id,
            title: input.title,
            episodes: input.episodes,
        };
        inner.items.insert(show.id, show.clone());
        Ok(show)
    }

    /// The stored show with this id, if there is one.
    pub fn get(&self, id: u64) -> Option<Show> {
        self.inner.lock().unwrap().items.get(&id).cloned()
    }
}

/// `POST /shows`: validate, store, answer `201` with the new show.
pub async fn create_show(
    State(store): State<Arc<ShowStore>>,
    ApiJson(input): ApiJson<NewShow>,
) -> Result<(StatusCode, Json<Show>), ApiError> {
    validate_new_show(&input)?;
    Ok((StatusCode::CREATED, Json(store.insert(input)?)))
}

/// `GET /shows/{id}`: the show, or `NotFound` with the message
/// `show {id} not found`.
pub async fn get_show(
    State(store): State<Arc<ShowStore>>,
    ApiPath(id): ApiPath<u64>,
) -> Result<Json<Show>, ApiError> {
    store
        .get(id)
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("show {id} not found")))
}

/// `GET /simulate-failure`: always fails with an `Internal` error whose text
/// looks like a leaked secret. A stand-in for a database going away (the real
/// ones start in module 5), so you can watch what a client is, and is not,
/// told.
pub async fn simulate_failure() -> Result<&'static str, ApiError> {
    Err(ApiError::Internal(
        "connect to postgres://anime:hunter2@db.internal:5432 refused".to_string(),
    ))
}

/// Fallback for a path no route matches: `NotFound` with the message
/// `no route for {path}`.
pub async fn route_not_found(uri: Uri) -> ApiError {
    ApiError::NotFound(format!("no route for {}", uri.path()))
}

/// Fallback for a known path with a method nobody registered.
pub async fn method_not_allowed() -> ApiError {
    ApiError::MethodNotAllowed
}

/// The route table:
///
/// | Route | Handler |
/// |---|---|
/// | `POST /shows` | [`create_show`] |
/// | `GET /shows/{id}` | [`get_show`] |
/// | `GET /simulate-failure` | [`simulate_failure`] |
///
/// A path that matches nothing goes to [`route_not_found`]. A known path with
/// the wrong method goes to [`method_not_allowed`].
pub fn app(store: Arc<ShowStore>) -> Router {
    Router::new()
        .route("/shows", post(create_show))
        .route("/shows/{id}", get(get_show))
        .route("/simulate-failure", get(simulate_failure))
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(store)
}
