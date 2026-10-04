use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct AppState {
    pub counter: Arc<Mutex<i64>>,
}

#[derive(Debug, Serialize)]
pub struct CounterResponse {
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

pub async fn hello() -> &'static str {
    "Hello, world!"
}

pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}

pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    let length = payload.message.len();
    Json(EchoResponse {
        message: payload.message,
        length,
    })
}

pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let count = *state.counter.lock().unwrap();
    Json(CounterResponse { count })
}

pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let mut guard = state.counter.lock().unwrap();
    *guard += 1;
    Json(CounterResponse { count: *guard })
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub q: String,
    pub limit: u32,
}

pub async fn search(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    Json(SearchResponse {
        q: params.q,
        limit: params.limit.unwrap_or(10),
    })
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/echo", post(echo))
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .route("/search", get(search))
        .with_state(state)
}
