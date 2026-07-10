use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
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
    // Bind the dereferenced value to a local before dropping `guard`,
    // rather than returning `*guard` as a bare tail expression — with the
    // latter, `guard`'s drop and the read can land in an order that trips
    // the borrow checker (E0597) once a function's *only* local is the
    // guard itself. Binding first sidesteps the question entirely.
    let count = *guard;
    drop(guard);
    Json(CounterResponse { count })
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/echo", post(echo))
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .with_state(state)
}
