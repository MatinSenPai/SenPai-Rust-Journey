//! Exercises for 3.2.1 — Routing, handlers, extractors.
//!
//! `hello` is given. You implement `greet`, `echo`, `get_counter`,
//! `increment_counter` and `app`; each doc comment is the whole spec.
//! The "Build" rung (a `/search` route with `Query<T>`) is yours to add.

use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Shared state. Every handler that declares `State<AppState>` receives a
/// clone of the value passed to `Router::with_state`. Cloning copies the
/// `Arc`, not the counter, so every clone sees the same number.
#[derive(Clone, Default)]
pub struct AppState {
    pub counter: Arc<Mutex<i64>>,
}

/// JSON body of the two counter routes: `{"count":N}`.
#[derive(Debug, Serialize)]
pub struct CounterResponse {
    pub count: i64,
}

/// JSON body `POST /echo` accepts: `{"message":"..."}`.
#[derive(Debug, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

/// JSON body `POST /echo` returns: `{"message":"...","length":N}`.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

/// `GET /` — given. No extractors, status 200, plain text `Hello, world!`.
pub async fn hello() -> &'static str {
    "Hello, world!"
}

/// `GET /greet/{name}` — status 200, plain-text body `Hello, NAME!` where
/// NAME is the path segment exactly as extracted (`/greet/senpai` gives
/// `Hello, senpai!`).
pub async fn greet(Path(name): Path<String>) -> String {
    todo!("return the greeting for `name`: the text Hello, NAME! with the name filled in")
}

/// `POST /echo` — request body `{"message":"hi"}`, status 200, JSON response
/// `{"message":"hi","length":2}`. `length` is the number of BYTES in the
/// message (`str::len`), not the number of characters. Bodies that are not
/// valid JSON, or lack `message`, never reach this function: the `Json`
/// extractor rejects them first.
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    todo!("answer with the same message and its byte length, as an EchoResponse in JSON")
}

/// `GET /counter` — status 200, JSON `{"count":N}` with the current value.
/// A fresh `AppState` holds 0, so the first call answers `{"count":0}`.
/// Does not change the counter.
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    todo!("report the shared counter's current value without changing it")
}

/// `POST /counter/increment` — adds 1 to the shared counter and answers
/// status 200, JSON `{"count":N}` with the value AFTER the increment (the
/// first call answers `{"count":1}`). Release the lock before returning.
pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    todo!("add one to the shared counter and report the new value")
}

/// Builds the router. Exactly these routes, and no others:
///
/// | method | path                 | handler             |
/// |--------|----------------------|---------------------|
/// | GET    | `/`                  | `hello`             |
/// | GET    | `/greet/{name}`      | `greet`             |
/// | POST   | `/echo`              | `echo`              |
/// | GET    | `/counter`           | `get_counter`       |
/// | POST   | `/counter/increment` | `increment_counter` |
///
/// `state` must be attached so the `State<AppState>` handlers work. An
/// unknown path answers 404; a known path with another method answers 405.
pub fn app(state: AppState) -> Router {
    todo!("register the five routes from the table above and attach `state`")
}
