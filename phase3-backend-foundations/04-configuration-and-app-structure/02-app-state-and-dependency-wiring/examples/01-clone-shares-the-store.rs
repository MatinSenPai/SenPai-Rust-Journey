//! `AppState` is cloned for every request, so a clone must be cheap and must
//! point at the same data. `Arc` gives both: cloning copies a pointer and
//! bumps a counter; the store behind it is not copied.
//!
//!     cargo run -p p3-04-02-app-state-and-dependency-wiring --example 01-clone-shares-the-store

use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct AppState {
    titles: Arc<Mutex<Vec<String>>>,
}

fn main() {
    let state = AppState::default();
    println!(
        "handles after creating the state: {}",
        Arc::strong_count(&state.titles)
    );

    let per_request = state.clone();
    println!(
        "handles after one clone:          {}",
        Arc::strong_count(&state.titles)
    );

    per_request
        .titles
        .lock()
        .unwrap()
        .push("Frieren".to_string());
    println!(
        "seen through the original:        {:?}",
        state.titles.lock().unwrap()
    );
    println!(
        "same allocation:                  {}",
        Arc::ptr_eq(&state.titles, &per_request.titles)
    );

    drop(per_request);
    println!(
        "handles after the clone is gone:  {}",
        Arc::strong_count(&state.titles)
    );
}
