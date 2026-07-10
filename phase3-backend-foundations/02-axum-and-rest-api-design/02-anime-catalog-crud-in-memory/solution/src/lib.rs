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
    pub rating: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnime {
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateAnime {
    pub title: Option<String>,
    pub status: Option<WatchStatus>,
    pub rating: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimeError {
    NotFound,
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
        validate_rating(input.rating)?;

        let mut inner = self.inner.lock().unwrap();
        inner.next_id += 1;
        let id = inner.next_id;
        let anime = Anime {
            id,
            title: input.title,
            status: input.status,
            rating: input.rating,
        };
        inner.items.insert(id, anime.clone());
        Ok(anime)
    }

    pub fn get(&self, id: u64) -> Result<Anime, AnimeError> {
        self.inner
            .lock()
            .unwrap()
            .items
            .get(&id)
            .cloned()
            .ok_or(AnimeError::NotFound)
    }

    pub fn list(&self) -> Vec<Anime> {
        let inner = self.inner.lock().unwrap();
        let mut items: Vec<Anime> = inner.items.values().cloned().collect();
        items.sort_by_key(|a| a.id);
        items
    }

    pub fn update(&self, id: u64, input: UpdateAnime) -> Result<Anime, AnimeError> {
        validate_rating(input.rating)?;

        let mut inner = self.inner.lock().unwrap();
        let anime = inner.items.get_mut(&id).ok_or(AnimeError::NotFound)?;
        if let Some(title) = input.title {
            anime.title = title;
        }
        if let Some(status) = input.status {
            anime.status = status;
        }
        if input.rating.is_some() {
            anime.rating = input.rating;
        }
        Ok(anime.clone())
    }

    pub fn delete(&self, id: u64) -> Result<Anime, AnimeError> {
        self.inner
            .lock()
            .unwrap()
            .items
            .remove(&id)
            .ok_or(AnimeError::NotFound)
    }
}

pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    let anime = store.create(input)?;
    Ok((StatusCode::CREATED, Json(anime)))
}

pub async fn list_anime(State(store): State<Arc<AnimeStore>>) -> Json<Vec<Anime>> {
    Json(store.list())
}

pub async fn get_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<Json<Anime>, AnimeError> {
    store.get(id).map(Json)
}

pub async fn update_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAnime>,
) -> Result<Json<Anime>, AnimeError> {
    store.update(id, input).map(Json)
}

pub async fn delete_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, AnimeError> {
    store.delete(id)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn app(store: Arc<AnimeStore>) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .route(
            "/anime/{id}",
            get(get_anime).patch(update_anime).delete(delete_anime),
        )
        .with_state(store)
}
