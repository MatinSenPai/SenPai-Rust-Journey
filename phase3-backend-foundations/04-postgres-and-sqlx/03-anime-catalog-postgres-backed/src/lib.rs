//! The exact same `Anime`/`WatchStatus`/`CreateAnime`/`UpdateAnime` domain as
//! `phase3-backend-foundations/02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory`,
//! rebuilt against real Postgres instead of a `Mutex<HashMap<...>>`. See
//! `README.md` for what actually changes (and what doesn't) between the two.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
}

impl WatchStatus {
    fn as_str(&self) -> &'static str {
        match self {
            WatchStatus::Watching => "watching",
            WatchStatus::Completed => "completed",
            WatchStatus::PlanToWatch => "plan_to_watch",
            WatchStatus::Dropped => "dropped",
        }
    }

    /// Parses a `p3_04_03_anime.status` column value back into a
    /// `WatchStatus`. Only ever fails if the database contains a value
    /// this crate itself never wrote — which, since `as_str`'s four
    /// variants are the only strings ever inserted, should be
    /// unreachable in practice; still returned as a `Result` (rather than
    /// panicking) because "trust but verify" is cheap here and a corrupt
    /// row shouldn't be able to crash the whole process.
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "watching" => Ok(WatchStatus::Watching),
            "completed" => Ok(WatchStatus::Completed),
            "plan_to_watch" => Ok(WatchStatus::PlanToWatch),
            "dropped" => Ok(WatchStatus::Dropped),
            other => Err(format!("unrecognized watch status in database: {other:?}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: i64,
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnime {
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<i16>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateAnime {
    pub title: Option<String>,
    pub status: Option<WatchStatus>,
    pub rating: Option<i16>,
}

#[derive(Debug)]
pub enum AnimeError {
    NotFound,
    InvalidRating(i16),
    /// Anything that went wrong talking to Postgres, or a row that came
    /// back with data this crate didn't put there itself (see
    /// `WatchStatus::parse`) — the same "wrap the underlying error as a
    /// String, don't leak `sqlx::Error` past this module" shape as
    /// `taskforge_core::JobError::Storage`.
    Storage(String),
}

impl IntoResponse for AnimeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
            AnimeError::InvalidRating(r) => (
                StatusCode::BAD_REQUEST,
                format!("rating must be between 1 and 10, got {r}"),
            ),
            AnimeError::Storage(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

fn validate_rating(rating: Option<i16>) -> Result<(), AnimeError> {
    match rating {
        Some(r) if !(1..=10).contains(&r) => Err(AnimeError::InvalidRating(r)),
        _ => Ok(()),
    }
}

/// Backed by a real `PgPool` instead of `Mutex<HashMap<...>>` — the same
/// public shape as the in-memory `AnimeStore`, so `app()` below and every
/// handler reads identically to module 2's version.
pub struct AnimeStore {
    pool: PgPool,
}

impl AnimeStore {
    pub fn new(pool: PgPool) -> Self {
        AnimeStore { pool }
    }

    pub async fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
        todo!(
            "validate_rating(input.rating)?; then run \
             sqlx::query(\"INSERT INTO p3_04_03_anime (title, status, rating) VALUES ($1, $2, \
             $3) RETURNING id\").bind(&input.title).bind(input.status.as_str())\
             .bind(input.rating).fetch_one(&self.pool).await, mapping any sqlx::Error to \
             AnimeError::Storage(e.to_string()) via .map_err(...)?; extract the new id with \
             .get::<i64, _>(\"id\") (or query_scalar for just the id); build and return an Anime \
             from the new id + input.title + input.status + input.rating: {{}}"
        )
    }

    pub async fn get(&self, id: i64) -> Result<Anime, AnimeError> {
        todo!(
            "query \"SELECT id, title, status, rating FROM p3_04_03_anime WHERE id = $1\" with \
             sqlx::query_as::<_, (i64, String, String, Option<i16>)>(...).bind(id)\
             .fetch_optional(&self.pool).await, mapping sqlx::Error to AnimeError::Storage; \
             .ok_or(AnimeError::NotFound)? the Option; then WatchStatus::parse the status \
             string (mapping its Err(String) to AnimeError::Storage) and build the Anime: {{}}"
        )
    }

    pub async fn list(&self) -> Result<Vec<Anime>, AnimeError> {
        todo!(
            "query \"SELECT id, title, status, rating FROM p3_04_03_anime ORDER BY id\" with \
             query_as::<_, (i64, String, String, Option<i16>)>(...).fetch_all(&self.pool).await, \
             mapping sqlx::Error to AnimeError::Storage; then map each row through \
             WatchStatus::parse (propagating any parse error as AnimeError::Storage) into a \
             Vec<Anime>, using .collect::<Result<Vec<_>, _>>()? to short-circuit on the first \
             parse failure: {{}}"
        )
    }

    pub async fn update(&self, id: i64, input: UpdateAnime) -> Result<Anime, AnimeError> {
        todo!(
            "validate_rating(input.rating)?; call self.get(id).await? first to fetch the \
             current row (this also gives you NotFound for free if it doesn't exist); build the \
             merged values (input.title.unwrap_or(current.title), \
             input.status.unwrap_or(current.status), input.rating.or(current.rating)); run \
             \"UPDATE p3_04_03_anime SET title = $1, status = $2, rating = $3 WHERE id = $4\" \
             with those merged values bound in order, mapping sqlx::Error to AnimeError::Storage; \
             then return an Anime built from the merged values + id: {{}}"
        )
    }

    pub async fn delete(&self, id: i64) -> Result<Anime, AnimeError> {
        todo!(
            "call self.get(id).await? first (NotFound for free if missing, and gives you the \
             full Anime to return on success); then run \"DELETE FROM p3_04_03_anime WHERE id = \
             $1\".bind(id).execute(&self.pool).await, mapping sqlx::Error to AnimeError::Storage; \
             return Ok(the anime fetched above): {{}}"
        )
    }
}

pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    todo!("call store.create(input).await?; return Ok((StatusCode::CREATED, Json(anime))): {{}}")
}

pub async fn list_anime(
    State(store): State<Arc<AnimeStore>>,
) -> Result<Json<Vec<Anime>>, AnimeError> {
    todo!("store.list().await.map(Json) : {{}}")
}

pub async fn get_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("store.get(id).await.map(Json) : {{}}")
}

pub async fn update_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateAnime>,
) -> Result<Json<Anime>, AnimeError> {
    todo!("store.update(id, input).await.map(Json) : {{}}")
}

pub async fn delete_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AnimeError> {
    todo!(
        "call store.delete(id).await? to make sure it existed, discard the returned Anime, \
         then return Ok(StatusCode::NO_CONTENT): {{}}"
    )
}

pub fn app(store: Arc<AnimeStore>) -> Router {
    todo!(
        "Router::new() \
            .route(\"/anime\", get(list_anime).post(create_anime)) \
            .route(\"/anime/{{id}}\", get(get_anime).patch(update_anime).delete(delete_anime)) \
            .with_state(store)"
    )
}
