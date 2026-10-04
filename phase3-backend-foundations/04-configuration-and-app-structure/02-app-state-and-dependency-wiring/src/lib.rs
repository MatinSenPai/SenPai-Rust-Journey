//! Exercises for 3.4.2 — Application state and dependency wiring.
//!
//! A tiny watch log: `POST /watch` records that you watched something, and
//! `GET /watch/recent` lists what you watched lately. "Lately" depends on the
//! current time, so the time comes from a [`Clock`] that lives in the
//! application state, which is what lets the tests drive it with a fake. The
//! README explains each decision.
//!
//! Work from the bottom of the stack up: the clocks, then the store, then the
//! state and handlers. `tests/clock_test.rs` and `tests/store_test.rs` need
//! no HTTP at all; `tests/api_test.rs` drives the whole router with `oneshot`.

use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// A source of "now", in whole seconds since the Unix epoch.
///
/// `Send + Sync` are supertraits on purpose: they are what lets an
/// `Arc<dyn Clock>` live in the application state.
pub trait Clock: Send + Sync {
    /// The current time as whole seconds since 1970-01-01 00:00:00 UTC.
    fn now(&self) -> u64;
}

/// The real clock: reads the operating system's wall clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

/// Returns the number of whole seconds between the Unix epoch and the moment
/// of the call, read from the system clock. A system clock set before 1970
/// answers `0`; it never panics.
impl Clock for SystemClock {
    fn now(&self) -> u64 {
        todo!("the whole seconds since the Unix epoch on the system clock, or 0 if it is set before 1970")
    }
}

/// A clock that only moves when a test says so. All methods take `&self`, so
/// a test can keep one `Arc<FakeClock>` and hand a clone of it to the
/// application state.
#[derive(Debug, Default)]
pub struct FakeClock {
    secs: AtomicU64,
}

impl FakeClock {
    /// A fake clock that reads `secs` until it is moved.
    pub fn at(secs: u64) -> Self {
        todo!("a fake clock whose time starts at secs")
    }

    /// Moves the clock forward by `secs` seconds. Every holder of the same
    /// `FakeClock` (through any `Arc`) sees the new time on its next read.
    pub fn advance(&self, secs: u64) {
        todo!("move this clock forward by secs seconds")
    }
}

/// Answers the time the fake clock was last set to by [`FakeClock::at`],
/// plus everything added by [`FakeClock::advance`] since.
impl Clock for FakeClock {
    fn now(&self) -> u64 {
        todo!("the fake clock's current time")
    }
}

/// One recorded viewing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Watch {
    /// Starts at `1` and goes up by one per entry; never reused.
    pub id: u64,
    pub title: String,
    /// When it was recorded, in seconds since the Unix epoch.
    pub watched_at: u64,
}

/// The in-memory log of viewings. It knows nothing about HTTP and nothing
/// about clocks: the caller says when each entry happened.
#[derive(Debug, Default)]
pub struct WatchStore {
    entries: Mutex<Vec<Watch>>,
}

impl WatchStore {
    /// Records a viewing of `title` at time `at` and returns the new entry.
    /// The first entry gets id `1`, the next `2`, and so on. `title` is stored
    /// exactly as given.
    pub fn add(&self, title: &str, at: u64) -> Watch {
        todo!("record a viewing of title at time at, with the next id, and return it")
    }

    /// Every entry whose `watched_at` is greater than or equal to `cutoff`, in
    /// the order they were added. An entry exactly at `cutoff` is included.
    pub fn since(&self, cutoff: u64) -> Vec<Watch> {
        todo!("copies of the entries at or after cutoff, in the order they were added")
    }

    /// How many entries have been recorded in total.
    pub fn count(&self) -> usize {
        todo!("the total number of entries recorded")
    }
}

/// The settings the handlers read. Plain data; in a real service the loader
/// from 3.4.1 would fill it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// How far back `GET /watch/recent` looks, in seconds.
    pub recent_window_secs: u64,
    /// The longest title `POST /watch` accepts, counted in characters.
    pub max_title_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recent_window_secs: 3600,
            max_title_len: 50,
        }
    }
}

/// Everything the handlers share. Cloning it clones three cheap handles, not
/// the data behind them. `#[derive(FromRef)]` lets a handler ask for any one
/// field with `State<FieldType>` instead of taking the whole struct.
#[derive(Clone, FromRef)]
pub struct AppState {
    pub clock: Arc<dyn Clock>,
    pub store: Arc<WatchStore>,
    pub config: Config,
}

impl AppState {
    /// A state with the given clock and settings and a new, empty store.
    pub fn new(clock: Arc<dyn Clock>, config: Config) -> Self {
        todo!("an AppState holding this clock, this config and a new empty store")
    }
}

/// The JSON body of `POST /watch`.
#[derive(Debug, Deserialize)]
pub struct NewWatch {
    pub title: String,
}

/// `POST /watch` — records a viewing, stamped with the clock's current time.
///
/// The title is trimmed of surrounding whitespace first (the stored title is
/// the trimmed one). A title that is empty after trimming, or longer than
/// `config.max_title_len` characters, answers `422 Unprocessable Entity` with
/// the body text `invalid title`, and nothing is recorded. Otherwise answers
/// `201 Created` with the new [`Watch`] as JSON.
pub async fn add_watch(
    State(clock): State<Arc<dyn Clock>>,
    State(store): State<Arc<WatchStore>>,
    State(config): State<Config>,
    Json(input): Json<NewWatch>,
) -> Result<(StatusCode, Json<Watch>), (StatusCode, &'static str)> {
    todo!(
        "record the trimmed title at the clock's time and answer 201 with it, or 422 invalid title"
    )
}

/// `GET /watch/recent` — answers `200` with a JSON array of the entries
/// recorded within the last `config.recent_window_secs` seconds, measured from
/// the clock's current time, in the order they were added. An entry exactly
/// `recent_window_secs` old still counts. A clock reading smaller than the
/// window counts everything.
pub async fn recent_watches(
    State(clock): State<Arc<dyn Clock>>,
    State(store): State<Arc<WatchStore>>,
    State(config): State<Config>,
) -> Json<Vec<Watch>> {
    todo!("the entries recorded within the configured window before the clock's now, as JSON")
}

/// The router the tests drive with `oneshot`: `POST /watch` and
/// `GET /watch/recent`, with `state` attached. Any other method on those
/// paths answers `405 Method Not Allowed`.
pub fn app(state: AppState) -> Router {
    todo!("a router with the two routes, sharing state")
}
