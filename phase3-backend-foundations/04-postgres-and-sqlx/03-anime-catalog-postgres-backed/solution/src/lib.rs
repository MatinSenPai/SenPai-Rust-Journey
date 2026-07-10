//! The exact same `Anime`/`WatchStatus`/`CreateAnime`/`UpdateAnime` domain as
//! `phase3-backend-foundations/02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory`,
//! rebuilt against real Postgres instead of a `Mutex<HashMap<...>>`. See
//! `../README.md` and `SOLUTION.md` for what actually changes (and what
//! doesn't) between the two.

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

pub struct AnimeStore {
    pool: PgPool,
}

impl AnimeStore {
    pub fn new(pool: PgPool) -> Self {
        AnimeStore { pool }
    }

    pub async fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
        validate_rating(input.rating)?;

        let id: i64 = sqlx::query_scalar(
            "INSERT INTO p3_04_03_anime (title, status, rating) VALUES ($1, $2, $3) \
             RETURNING id",
        )
        .bind(&input.title)
        .bind(input.status.as_str())
        .bind(input.rating)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnimeError::Storage(e.to_string()))?;

        Ok(Anime {
            id,
            title: input.title,
            status: input.status,
            rating: input.rating,
        })
    }

    pub async fn get(&self, id: i64) -> Result<Anime, AnimeError> {
        let row: Option<(i64, String, String, Option<i16>)> =
            sqlx::query_as("SELECT id, title, status, rating FROM p3_04_03_anime WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AnimeError::Storage(e.to_string()))?;

        let (id, title, status, rating) = row.ok_or(AnimeError::NotFound)?;
        let status = WatchStatus::parse(&status).map_err(AnimeError::Storage)?;
        Ok(Anime {
            id,
            title,
            status,
            rating,
        })
    }

    pub async fn list(&self) -> Result<Vec<Anime>, AnimeError> {
        let rows: Vec<(i64, String, String, Option<i16>)> =
            sqlx::query_as("SELECT id, title, status, rating FROM p3_04_03_anime ORDER BY id")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AnimeError::Storage(e.to_string()))?;

        rows.into_iter()
            .map(|(id, title, status, rating)| {
                let status = WatchStatus::parse(&status).map_err(AnimeError::Storage)?;
                Ok(Anime {
                    id,
                    title,
                    status,
                    rating,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn update(&self, id: i64, input: UpdateAnime) -> Result<Anime, AnimeError> {
        validate_rating(input.rating)?;

        let current = self.get(id).await?;
        let title = input.title.unwrap_or(current.title);
        let status = input.status.unwrap_or(current.status);
        let rating = input.rating.or(current.rating);

        sqlx::query("UPDATE p3_04_03_anime SET title = $1, status = $2, rating = $3 WHERE id = $4")
            .bind(&title)
            .bind(status.as_str())
            .bind(rating)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AnimeError::Storage(e.to_string()))?;

        Ok(Anime {
            id,
            title,
            status,
            rating,
        })
    }

    pub async fn delete(&self, id: i64) -> Result<Anime, AnimeError> {
        let current = self.get(id).await?;

        sqlx::query("DELETE FROM p3_04_03_anime WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AnimeError::Storage(e.to_string()))?;

        Ok(current)
    }
}

pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    let anime = store.create(input).await?;
    Ok((StatusCode::CREATED, Json(anime)))
}

pub async fn list_anime(
    State(store): State<Arc<AnimeStore>>,
) -> Result<Json<Vec<Anime>>, AnimeError> {
    store.list().await.map(Json)
}

pub async fn get_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
) -> Result<Json<Anime>, AnimeError> {
    store.get(id).await.map(Json)
}

pub async fn update_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateAnime>,
) -> Result<Json<Anime>, AnimeError> {
    store.update(id, input).await.map(Json)
}

pub async fn delete_anime(
    State(store): State<Arc<AnimeStore>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AnimeError> {
    store.delete(id).await?;
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
