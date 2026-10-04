//! DELIBERATELY BROKEN — expected: E0277
//!
//! `Json<T>` reads the request body, and a body can be read only once, so
//! `axum` accepts it only as a handler's LAST parameter. Here it comes first.
//!
//!     cargo build -p p3-02-01-routing-handlers-extractors --example 06-body-extractor-not-last-broken --features broken

use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

#[derive(Clone, Default)]
struct AppState {
    notes: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize)]
struct NewNote {
    text: String,
}

async fn add_note(Json(note): Json<NewNote>, State(state): State<AppState>) -> String {
    let mut notes = state.notes.lock().unwrap();
    notes.push(note.text);
    format!("{} notes", notes.len())
}

fn main() {
    let _app: Router = Router::new()
        .route("/notes", post(add_note))
        .with_state(AppState::default());
}
