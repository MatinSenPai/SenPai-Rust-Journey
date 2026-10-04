//! Exercises for 3.2.3 — Anime catalog CRUD (in-memory).
//!
//! Two layers. `AnimeStore` is plain Rust: it knows nothing about `axum`,
//! JSON or status codes, and `tests/store_test.rs` checks it with ordinary
//! function calls. The handlers below it are the thin HTTP edge: take the
//! request apart with extractors, call one store method, and turn the
//! `Result` into a response. `tests/api_test.rs` checks that edge through
//! `tower::ServiceExt::oneshot`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::header::HeaderName;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Where you are with a show. On the wire it is snake_case:
/// `"watching"`, `"completed"`, `"plan_to_watch"`, `"dropped"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
}

/// One catalog entry. As JSON:
/// `{"id":1,"title":"Frieren","status":"watching","rating":9}`, with
/// `"rating":null` when the show has no rating yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    /// `1..=10`, or `None` if not rated yet.
    pub rating: Option<u8>,
}

/// The body of `POST /anime`. `title` and `status` are required; `rating`
/// may be left out or sent as `null`. The server picks the id.
#[derive(Debug, Deserialize)]
pub struct CreateAnime {
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

/// The body of `PATCH /anime/{id}`: a partial update. Every field is
/// optional, and `None` (the field left out, or sent as `null`) means
/// "leave this field as it is", never "clear it".
#[derive(Debug, Deserialize, Default)]
pub struct UpdateAnime {
    pub title: Option<String>,
    pub status: Option<WatchStatus>,
    pub rating: Option<u8>,
}

/// Everything that can go wrong in the catalog, in the catalog's own terms.
/// Each variant becomes exactly one HTTP response (see the `IntoResponse`
/// impl below):
///
/// | Variant | Status | JSON body |
/// |---|---|---|
/// | `NotFound` | `404 Not Found` | `{"error":"anime not found"}` |
/// | `InvalidRating(r)` | `422 Unprocessable Entity` | `{"error":"rating must be between 1 and 10, got {r}"}` |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimeError {
    /// No anime has the requested id.
    NotFound,
    /// A rating arrived that is outside `1..=10`. Carries the bad value.
    InvalidRating(u8),
}

impl IntoResponse for AnimeError {
    /// Builds the response from the table on [`AnimeError`]: that status
    /// code, and a JSON object with a single `"error"` key holding that
    /// message, sent with `Content-Type: application/json`. For
    /// `InvalidRating(15)` the body is exactly
    /// `{"error":"rating must be between 1 and 10, got 15"}`.
    fn into_response(self) -> Response {
        todo!("the status code and JSON error body the table on AnimeError gives this variant")
    }
}

#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Anime>,
}

/// The in-memory catalog: every anime, plus the counter that hands out ids.
/// Both live behind one `Mutex`, so the router can share a single store
/// across every request (and every thread) through `Arc`. Gone when the
/// process exits; module 5 rebuilds the same shape on Postgres.
#[derive(Default)]
pub struct AnimeStore {
    inner: Mutex<StoreInner>,
}

/// `Ok(())` for no rating or a rating in `1..=10`; otherwise
/// `Err(InvalidRating(r))`. Given to you: use it wherever a rating arrives.
fn validate_rating(rating: Option<u8>) -> Result<(), AnimeError> {
    match rating {
        Some(r) if !(1..=10).contains(&r) => Err(AnimeError::InvalidRating(r)),
        _ => Ok(()),
    }
}

impl AnimeStore {
    /// Adds a new anime and returns it, id included.
    ///
    /// - The rating is checked first: outside `1..=10` gives
    ///   `Err(InvalidRating(r))`, and the store is left exactly as it was
    ///   (no id is used up by a rejected create).
    /// - Ids start at `1` and go up by one per successful create. An id is
    ///   never handed out twice, not even after its anime is deleted.
    pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
        todo!("store a new anime under the next unused id and return it, or InvalidRating")
    }

    /// A copy of the anime with this id, or `Err(NotFound)`.
    pub fn get(&self, id: u64) -> Result<Anime, AnimeError> {
        todo!("a copy of the stored anime with this id, or NotFound")
    }

    /// Copies of every stored anime, sorted by id, smallest first. An empty
    /// store gives an empty `Vec`. (A `HashMap` iterates in no particular
    /// order, so the sorting is on you.)
    pub fn list(&self) -> Vec<Anime> {
        todo!("every stored anime, sorted by id")
    }

    /// Applies a partial update and returns the anime as it now is.
    ///
    /// - The rating is checked first: an out-of-range rating gives
    ///   `Err(InvalidRating(r))` whether or not the id exists.
    /// - Then a missing id gives `Err(NotFound)`.
    /// - Otherwise each field that is `Some` in `input` replaces the stored
    ///   value, and each `None` leaves it alone.
    ///
    /// On any `Err`, nothing in the store changes.
    pub fn update(&self, id: u64, input: UpdateAnime) -> Result<Anime, AnimeError> {
        todo!("change only the fields input sets and return the result, or the error")
    }

    /// Removes the anime with this id and returns what was removed, or
    /// `Err(NotFound)` if there was nothing to remove.
    pub fn delete(&self, id: u64) -> Result<Anime, AnimeError> {
        todo!("remove the anime with this id and return it, or NotFound")
    }
}

/// `POST /anime`, body [`CreateAnime`].
///
/// Success: `201 Created`, a `Location` header naming the new resource
/// (`/anime/1` for id 1), and the created anime as the JSON body.
/// Failure: the store's `AnimeError` (a bad rating is a `422`).
pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, [(HeaderName, String); 1], Json<Anime>), AnimeError> {
    todo!("create the anime; answer 201 with its Location and the anime as JSON")
}

/// `GET /anime`: `200 OK` with a JSON array of every anime, sorted by id
/// (`[]` when the catalog is empty).
pub async fn list_anime(State(store): State<Arc<AnimeStore>>) -> Json<Vec<Anime>> {
    todo!("every anime as a JSON array")
}

/// `GET /anime/{id}`: `200 OK` with the anime as JSON, or the `404` from
/// `AnimeError::NotFound`.
pub async fn get_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("the anime with this id as JSON, or the store's error")
}

/// `PATCH /anime/{id}`, body [`UpdateAnime`]: `200 OK` with the anime as it
/// is after the update, or the store's `AnimeError` (`404` for a missing
/// id, `422` for a bad rating).
pub async fn update_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAnime>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("apply the partial update and return the updated anime as JSON, or the store's error")
}

/// `DELETE /anime/{id}`: `204 No Content` with an empty body, or the `404`
/// from `AnimeError::NotFound` when there was nothing to delete.
pub async fn delete_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AnimeError> {
    todo!("delete the anime and answer 204, or the store's error")
}

/// The route table, sharing `store` with every handler:
///
/// | Path | Method | Handler |
/// |---|---|---|
/// | `/anime` | `GET` | [`list_anime`] |
/// | `/anime` | `POST` | [`create_anime`] |
/// | `/anime/{id}` | `GET` | [`get_anime`] |
/// | `/anime/{id}` | `PATCH` | [`update_anime`] |
/// | `/anime/{id}` | `DELETE` | [`delete_anime`] |
///
/// Two routes, one per path, each answering several methods.
pub fn app(store: Arc<AnimeStore>) -> Router {
    todo!("the two routes from the table above, with the store as shared state")
}
