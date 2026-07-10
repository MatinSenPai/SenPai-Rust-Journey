//! Design a URL shortener — the classic first system-design interview
//! question, built for real. See `README.md` for the requirements walkthrough
//! (what a naive design looks like, then where it breaks) before touching
//! any `todo!()` here.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Encodes a non-negative integer id as a base62 string — the same idea as
/// hex, just with a 62-character alphabet instead of 16, so the same range
/// of ids fits in far fewer characters (a 64-bit id needs at most ~11
/// base62 characters, vs 20 decimal digits). This is what turns
/// row id `125` into a short, URL-safe code instead of a long number.
pub fn base62_encode(mut n: i64) -> String {
    todo!(
        "if n == 0, return \"0\".to_string(); otherwise build a Vec<u8> of alphabet bytes by \
         repeatedly taking n % 62 as an index into BASE62_ALPHABET, pushing that byte, then n \
         /= 62, looping while n > 0; reverse the Vec (you built it least-significant-digit \
         first) and String::from_utf8(...).unwrap() it: {{}}"
    )
}

/// The inverse of `base62_encode` — not used by the service itself (lookups
/// go straight to the database by the short_code string), but a natural
/// pure-function pairing for testing the encoding round-trips correctly.
pub fn base62_decode(s: &str) -> Option<i64> {
    todo!(
        "fold over s.bytes(): for each byte, find its index in BASE62_ALPHABET \
         (BASE62_ALPHABET.iter().position(|&b| b == byte)?), and accumulate \
         n = n * 62 + index as i64, starting from n = 0i64; return Some(n) at the end \
         (the ? on .position(...) inside a function returning Option lets a byte that isn't in \
         the alphabet short-circuit the whole thing to None): {{}}"
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortUrl {
    pub short_code: String,
    pub original_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrlStats {
    pub short_code: String,
    pub original_url: String,
    pub click_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

#[derive(Debug)]
pub enum UrlShortenerError {
    InvalidUrl(String),
    NotFound,
    Storage(String),
}

impl IntoResponse for UrlShortenerError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            UrlShortenerError::InvalidUrl(message) => (StatusCode::BAD_REQUEST, message),
            UrlShortenerError::NotFound => {
                (StatusCode::NOT_FOUND, "short code not found".to_string())
            }
            UrlShortenerError::Storage(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

fn validate_url(url: &str) -> Result<(), UrlShortenerError> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(UrlShortenerError::InvalidUrl(
            "url must start with http:// or https://".to_string(),
        ))
    }
}

/// Backed by a Postgres sequence (for collision-free ids to base62-encode)
/// plus one table. See `README.md` for why a sequence, not `BIGSERIAL`, is
/// used here — the short code needs the id *before* the row is inserted.
pub struct UrlShortenerStore {
    pool: PgPool,
}

impl UrlShortenerStore {
    pub fn new(pool: PgPool) -> Self {
        UrlShortenerStore { pool }
    }

    pub async fn shorten(&self, original_url: &str) -> Result<ShortUrl, UrlShortenerError> {
        validate_url(original_url)?;
        todo!(
            "pull the next id with sqlx::query_scalar::<_, i64>(\"SELECT nextval\
             ('p5_07_01_urls_id_seq')\").fetch_one(&self.pool).await, mapping sqlx::Error to \
             UrlShortenerError::Storage(e.to_string()); base62_encode(id) to get short_code; \
             INSERT INTO p5_07_01_urls (id, short_code, original_url) VALUES ($1, $2, $3) \
             binding id, &short_code, original_url, .execute(&self.pool).await, same error \
             mapping; return Ok(ShortUrl {{ short_code, original_url: original_url.to_string() \
             }}): {{}}"
        )
    }

    /// Resolves a short code to its original URL, incrementing the click
    /// counter in the same statement — one atomic round trip instead of a
    /// separate read-then-write (see `05-security-and-auth-at-scale`'s
    /// sibling locking lesson for why a *separate* read-then-write on a
    /// shared counter would be a real race under concurrent traffic).
    pub async fn resolve(&self, short_code: &str) -> Result<Option<String>, UrlShortenerError> {
        todo!(
            "sqlx::query_scalar::<_, String>(\"UPDATE p5_07_01_urls SET click_count = \
             click_count + 1 WHERE short_code = $1 RETURNING original_url\").bind(short_code)\
             .fetch_optional(&self.pool).await, mapping sqlx::Error to \
             UrlShortenerError::Storage(e.to_string()): {{}}"
        )
    }

    pub async fn stats(&self, short_code: &str) -> Result<Option<UrlStats>, UrlShortenerError> {
        todo!(
            "sqlx::query_as::<_, (String, String, i64)>(\"SELECT short_code, original_url, \
             click_count FROM p5_07_01_urls WHERE short_code = $1\").bind(short_code)\
             .fetch_optional(&self.pool).await, mapping sqlx::Error to \
             UrlShortenerError::Storage(e.to_string()); map the Option<(String, String, i64)> \
             into Option<UrlStats> with the three fields in order: {{}}"
        )
    }
}

pub async fn shorten_url(
    State(store): State<Arc<UrlShortenerStore>>,
    Json(req): Json<ShortenRequest>,
) -> Result<(StatusCode, Json<ShortUrl>), UrlShortenerError> {
    let short_url = store.shorten(&req.url).await?;
    Ok((StatusCode::CREATED, Json(short_url)))
}

pub async fn redirect_to_original(
    State(store): State<Arc<UrlShortenerStore>>,
    Path(code): Path<String>,
) -> Result<Redirect, UrlShortenerError> {
    match store.resolve(&code).await? {
        Some(url) => Ok(Redirect::to(&url)),
        None => Err(UrlShortenerError::NotFound),
    }
}

pub async fn get_stats(
    State(store): State<Arc<UrlShortenerStore>>,
    Path(code): Path<String>,
) -> Result<Json<UrlStats>, UrlShortenerError> {
    store
        .stats(&code)
        .await?
        .map(Json)
        .ok_or(UrlShortenerError::NotFound)
}

pub fn app(store: Arc<UrlShortenerStore>) -> Router {
    Router::new()
        .route("/shorten", post(shorten_url))
        .route("/{code}", get(redirect_to_original))
        .route("/{code}/stats", get(get_stats))
        .with_state(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base62_round_trips_a_range_of_ids() {
        for id in [0i64, 1, 61, 62, 125, 12345, 999_999_999] {
            let encoded = base62_encode(id);
            assert_eq!(base62_decode(&encoded), Some(id));
        }
    }

    #[test]
    fn base62_encode_zero_is_the_single_digit_zero() {
        assert_eq!(base62_encode(0), "0");
    }

    #[test]
    fn base62_encode_is_shorter_than_decimal_for_large_ids() {
        let id = 999_999_999_999i64;
        assert!(base62_encode(id).len() < id.to_string().len());
    }
}
