use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// Shared state every handler that declares `State<AppState>` in its
/// signature receives a clone of. `Arc<Mutex<i64>>` mirrors the "shared,
/// continuously-updated state" case from the channels lesson — several
/// concurrent requests may hit `/counter/increment` at once, and the
/// `Mutex` is what keeps that safe.
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

/// No extractors at all — a handler can just take no arguments and return
/// something that implements `IntoResponse` (a `&'static str` does).
pub async fn hello() -> &'static str {
    "Hello, world!"
}

/// `Path<String>` pulls the `{name}` segment out of the route
/// (`/greet/{name}`) registered in `app` below.
pub async fn greet(Path(name): Path<String>) -> String {
    todo!("format!(\"Hello, {{name}}!\")")
}

/// `Json<EchoRequest>` deserializes the request body; returning
/// `Json<EchoResponse>` serializes the response body. If the request body
/// isn't valid JSON, or doesn't match `EchoRequest`'s shape, `axum` returns
/// an error response automatically — this function only ever runs with a
/// successfully-parsed `EchoRequest` in hand.
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    todo!(
        "build an EchoResponse from payload.message (length = payload.message.len()), \
         wrap it in Json(..) and return it"
    )
}

/// `State<AppState>` retrieves the `AppState` passed to `Router::with_state`
/// in `app`. Locking a `Mutex` inside a handler briefly blocks the running
/// task, but only for the few instructions it takes to read one `i64` —
/// short enough not to worry about for this lesson (module 4 revisits
/// "what should and shouldn't hold a lock across an `.await` point").
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    todo!(
        "lock state.counter (state.counter.lock().unwrap()), read the current value, \
         return Json(CounterResponse {{ count: *value }})"
    )
}

pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    todo!(
        "lock state.counter, add 1 to the value in place (*guard += 1), read the new \
         value out into a local variable, then return Json(CounterResponse {{ count: new_value }}) \
         — bind the dereferenced value to a local before returning it (see the README/gotchas: \
         returning `*guard` as the bare last expression while `guard` itself is about to be \
         dropped can trip the borrow checker)"
    )
}

/// Wires every handler onto a route and attaches `state` so every
/// `State<AppState>` extractor above has something to retrieve.
pub fn app(state: AppState) -> Router {
    todo!(
        "Router::new() \
            .route(\"/\", get(hello)) \
            .route(\"/greet/{{name}}\", get(greet)) \
            .route(\"/echo\", post(echo)) \
            .route(\"/counter\", get(get_counter)) \
            .route(\"/counter/increment\", post(increment_counter)) \
            .with_state(state)"
    )
}
