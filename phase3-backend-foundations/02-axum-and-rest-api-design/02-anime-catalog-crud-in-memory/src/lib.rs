use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    /// 1-10, or `None` if not yet rated.
    pub rating: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnime {
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

/// Every field optional — a `PATCH`-style partial update. `None` means
/// "leave this field alone," not "clear it."
#[derive(Debug, Deserialize, Default)]
pub struct UpdateAnime {
    pub title: Option<String>,
    pub status: Option<WatchStatus>,
    pub rating: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimeError {
    NotFound,
    /// The rating supplied was outside the valid `1..=10` range.
    InvalidRating(u8),
}

impl IntoResponse for AnimeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
            AnimeError::InvalidRating(r) => (
                StatusCode::BAD_REQUEST,
                format!("rating must be between 1 and 10, got {r}"),
            ),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Anime>,
}

/// An in-memory catalog: a `HashMap` behind a `Mutex`, gone the moment the
/// process exits. Deliberately the same shape module 4 rebuilds against
/// real Postgres — knows nothing about `axum` or HTTP, so it's fully
/// testable with plain function calls (see `tests/store_test.rs`).
#[derive(Default)]
pub struct AnimeStore {
    inner: Mutex<StoreInner>,
}

fn validate_rating(rating: Option<u8>) -> Result<(), AnimeError> {
    match rating {
        Some(r) if !(1..=10).contains(&r) => Err(AnimeError::InvalidRating(r)),
        _ => Ok(()),
    }
}

impl AnimeStore {
    pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
        todo!(
            "validate_rating(input.rating)?; lock self.inner; increment inner.next_id and use \
             that as the new id; build an Anime from input + the new id; insert a clone into \
             inner.items keyed by id; return Ok(anime)"
        )
    }

    pub fn get(&self, id: u64) -> Result<Anime, AnimeError> {
        todo!("lock self.inner, look up id in inner.items, .cloned(), .ok_or(AnimeError::NotFound)")
    }

    /// Every item, ordered by id (a `HashMap`'s natural iteration order is
    /// unspecified — sort explicitly so responses are deterministic, which
    /// matters both for tests and for any real client relying on stable
    /// ordering).
    pub fn list(&self) -> Vec<Anime> {
        todo!(
            "lock self.inner, collect inner.items.values().cloned() into a Vec<Anime>, \
             sort_by_key(|a| a.id), return it"
        )
    }

    pub fn update(&self, id: u64, input: UpdateAnime) -> Result<Anime, AnimeError> {
        todo!(
            "validate_rating(input.rating)?; lock self.inner; get_mut the entry for id \
             (.ok_or(AnimeError::NotFound)?); for each Some(field) in input, overwrite the \
             matching field on the stored Anime; return Ok(the updated Anime.clone())"
        )
    }

    pub fn delete(&self, id: u64) -> Result<Anime, AnimeError> {
        todo!("lock self.inner, inner.items.remove(&id).ok_or(AnimeError::NotFound)")
    }
}

pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    todo!(
        "call store.create(input)?; return Ok((StatusCode::CREATED, Json(anime))) — a POST that \
         creates a resource conventionally answers 201, not 200"
    )
}

pub async fn list_anime(State(store): State<Arc<AnimeStore>>) -> Json<Vec<Anime>> {
    todo!("Json(store.list())")
}

pub async fn get_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("store.get(id).map(Json)")
}

pub async fn update_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAnime>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("store.update(id, input).map(Json)")
}

pub async fn delete_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AnimeError> {
    todo!(
        "call store.delete(id)? to make sure it existed, discard the returned Anime, then \
         return Ok(StatusCode::NO_CONTENT) — a successful DELETE with nothing useful to hand \
         back in the body conventionally answers 204"
    )
}

/// `/anime` handles listing and creating; `/anime/{id}` handles fetching,
/// partially updating, and deleting *one* item — the same two-route shape
/// a DRF `ModelViewSet` generates for you automatically, built here by hand.
pub fn app(store: Arc<AnimeStore>) -> Router {
    todo!(
        "Router::new() \
            .route(\"/anime\", get(list_anime).post(create_anime)) \
            .route(\"/anime/{{id}}\", get(get_anime).patch(update_anime).delete(delete_anime)) \
            .with_state(store) \
         — .patch(..) and .delete(..) are chaining methods on the MethodRouter that get(..) \
         returns, same as .post(..) was on last lesson's routes, so no new imports needed \
         beyond the get already imported at the top of this file"
    )
}
