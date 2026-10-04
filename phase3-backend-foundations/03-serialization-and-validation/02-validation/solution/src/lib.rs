//! Exercises for 3.3.2 — Validation.
//!
//! `serde` decides whether a body has the right *shape*; `validator`
//! decides whether a well-shaped value obeys the *rules*. The rules live on
//! the types below as `#[validate(...)]` attributes. Your work is the part
//! around them: one custom rule, turning `validator`'s error tree into a
//! flat field-keyed map, and answering a `422` with it.
//!
//! `tests/validate_test.rs` checks the first two with plain calls (no
//! `axum`); `tests/api_test.rs` checks the HTTP edge through `oneshot`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::extract::{FromRequest, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

/// Who wrote the review. Both fields are checked when the review is.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Reviewer {
    #[validate(
        length(min = 3, max = 20, message = "handle must be 3 to 20 characters"),
        custom(function = "validate_handle")
    )]
    pub handle: String,
    #[validate(email)]
    pub email: String,
}

/// A note on one episode. `episode` starts at 1.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EpisodeNote {
    #[validate(range(min = 1))]
    pub episode: u32,
    #[validate(length(min = 1, max = 200, message = "note must be 1 to 200 characters"))]
    pub text: String,
}

/// The body of `POST /reviews`. `body` and `notes` may be left out.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewReview {
    #[validate(length(min = 1, max = 100, message = "title must be 1 to 100 characters"))]
    pub title: String,
    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    pub rating: u8,
    #[validate(length(max = 500, message = "body must be at most 500 characters"))]
    #[serde(default)]
    pub body: Option<String>,
    #[validate(nested)]
    pub reviewer: Reviewer,
    #[validate(nested)]
    #[serde(default)]
    pub notes: Vec<EpisodeNote>,
}

/// A custom rule for `Reviewer::handle`: letters, digits and `_` only.
///
/// - `Ok(())` when every character is an ASCII letter, an ASCII digit or
///   `_` (the empty string passes: length is another rule's job).
/// - Otherwise `Err` with code exactly `"handle_chars"` and message exactly
///   `"handle may only contain letters, digits and underscores"`.
pub fn validate_handle(handle: &str) -> Result<(), ValidationError> {
    if handle
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        Ok(())
    } else {
        Err(ValidationError::new("handle_chars")
            .with_message("handle may only contain letters, digits and underscores".into()))
    }
}

/// Flattens the error tree `validator` returns into `path -> messages`.
///
/// - A field with rule failures gets its own key: `"title"`.
/// - A failing nested struct prefixes its field names with the parent's
///   name and a dot: `"reviewer.email"`.
/// - A failing item in a nested `Vec` adds its index in brackets:
///   `"notes[0].text"`.
/// - Each message is the `ValidationError`'s `message` if it has one,
///   otherwise its `code`. Each key's messages are sorted ascending.
/// - An empty `ValidationErrors` gives an empty map.
///
/// The map is a `BTreeMap`, so keys come out sorted too.
pub fn flatten_errors(errors: &ValidationErrors) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    walk(errors, "", &mut out);
    out
}

fn walk(errors: &ValidationErrors, prefix: &str, out: &mut BTreeMap<String, Vec<String>>) {
    for (field, kind) in errors.errors() {
        let path = format!("{prefix}{field}");
        match kind {
            ValidationErrorsKind::Field(list) => {
                let mut msgs: Vec<String> = list
                    .iter()
                    .map(|e| e.message.as_ref().unwrap_or(&e.code).to_string())
                    .collect();
                msgs.sort();
                out.insert(path, msgs);
            }
            ValidationErrorsKind::Struct(inner) => walk(inner, &format!("{path}."), out),
            ValidationErrorsKind::List(items) => {
                for (i, inner) in items {
                    walk(inner, &format!("{path}[{i}]."), out);
                }
            }
        }
    }
}

/// What a handler can fail with.
///
/// `Validation` becomes `422 Unprocessable Entity` with the JSON body
/// `{"errors": <flatten_errors of the tree>}`, sent as
/// `Content-Type: application/json`. For a rating of `15` the body is
/// exactly `{"errors":{"rating":["rating must be between 1 and 10"]}}`.
#[derive(Debug)]
pub enum ApiError {
    Validation(ValidationErrors),
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::Validation(errors)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": flatten_errors(&errors) })),
            )
                .into_response(),
        }
    }
}

/// A review the server accepted.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StoredReview {
    pub id: u64,
    pub title: String,
    pub rating: u8,
    pub reviewer: String,
}

/// Every accepted review, in order. Ids start at 1. Given to you.
#[derive(Default)]
pub struct ReviewStore {
    items: Mutex<Vec<StoredReview>>,
}

impl ReviewStore {
    pub fn add(&self, review: NewReview) -> StoredReview {
        let mut items = self.items.lock().unwrap();
        let stored = StoredReview {
            id: items.len() as u64 + 1,
            title: review.title,
            rating: review.rating,
            reviewer: review.reviewer.handle,
        };
        items.push(stored.clone());
        stored
    }

    pub fn list(&self) -> Vec<StoredReview> {
        self.items.lock().unwrap().clone()
    }
}

/// Build: a `Json<T>` that also runs `T`'s validation rules.
///
/// - The body is read like `Json<T>` does. If that fails, the rejection is
///   `Json<T>`'s own, unchanged (`400`, `415`, or `422` for a wrong shape).
/// - If it succeeds, `value.validate()` runs; a failure answers the `422`
///   from [`ApiError`].
/// - Otherwise the handler gets the value.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;
        value
            .validate()
            .map_err(|e| ApiError::from(e).into_response())?;
        Ok(ValidatedJson(value))
    }
}

/// `POST /reviews`, body [`NewReview`] (read with `Json`).
///
/// Success: `201 Created` and the stored review as JSON. A review that
/// breaks any rule (nested ones included) is not stored, and answers the
/// `422` from [`ApiError`].
pub async fn create_review(
    State(store): State<Arc<ReviewStore>>,
    Json(input): Json<NewReview>,
) -> Result<(StatusCode, Json<StoredReview>), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, Json(store.add(input))))
}

/// `GET /reviews`: `200 OK` and every stored review as a JSON array.
pub async fn list_reviews(State(store): State<Arc<ReviewStore>>) -> Json<Vec<StoredReview>> {
    Json(store.list())
}

/// One route, `/reviews`: `GET` is [`list_reviews`], `POST` is
/// [`create_review`]. The store is shared with both handlers.
pub fn app(store: Arc<ReviewStore>) -> Router {
    Router::new()
        .route("/reviews", get(list_reviews).post(create_review))
        .with_state(store)
}
