//! A tiny "webtoon tracker" service: follow a webtoon at its current known
//! chapter, and a background job periodically checks whether a new chapter
//! has come out, logging when it has. See `README.md` for the tour.
//!
//! Everything in this file down to [`ChapterChecker`] and its
//! implementations is given, fully working. The exercise lives in four
//! spots: [`follow_webtoon`], [`check_webtoon`] (the two handler bodies
//! most worth writing yourself), [`check_all_webtoons`] (the interesting
//! "did anything change" loop the background job runs), and [`app`] (the
//! router wiring, same shape as Phase 3's anime catalog CRUD lesson).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// A webtoon someone is following, and the last chapter number we know
/// about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Webtoon {
    pub id: u64,
    pub title: String,
    pub current_chapter: u32,
}

/// Request body for `POST /webtoons`.
#[derive(Debug, Deserialize)]
pub struct FollowWebtoon {
    pub title: String,
    pub current_chapter: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebtoonError {
    NotFound,
}

impl IntoResponse for WebtoonError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            WebtoonError::NotFound => (StatusCode::NOT_FOUND, "webtoon not found".to_string()),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Webtoon>,
}

/// An in-memory, `Mutex`-guarded catalog of followed webtoons — the same
/// shape as `AnimeStore` from Phase 3's CRUD lesson
/// (`phase3-backend-foundations/02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory`).
/// The Phase 3 README's blurb about this side-quest name-drops Postgres —
/// that's the "if you kept building this for real" direction, previewing
/// module 4 of Phase 3. As a side-quest, this stays in-memory on purpose so
/// it never needs a live database to run or to grade.
#[derive(Default)]
pub struct WebtoonStore {
    inner: Mutex<StoreInner>,
}

impl WebtoonStore {
    /// Starts following a new webtoon, assigning it a fresh id.
    pub fn follow(&self, input: FollowWebtoon) -> Webtoon {
        let mut inner = self.inner.lock().unwrap();
        inner.next_id += 1;
        let id = inner.next_id;
        let webtoon = Webtoon {
            id,
            title: input.title,
            current_chapter: input.current_chapter,
        };
        inner.items.insert(id, webtoon.clone());
        webtoon
    }

    /// Looks up one followed webtoon by id.
    pub fn get(&self, id: u64) -> Option<Webtoon> {
        self.inner.lock().unwrap().items.get(&id).cloned()
    }

    /// Every followed webtoon, ordered by id (a `HashMap`'s iteration order
    /// is unspecified, so this sorts explicitly for deterministic
    /// responses — same reasoning as `AnimeStore::list`).
    pub fn list(&self) -> Vec<Webtoon> {
        let inner = self.inner.lock().unwrap();
        let mut items: Vec<Webtoon> = inner.items.values().cloned().collect();
        items.sort_by_key(|w| w.id);
        items
    }

    /// Overwrites the stored chapter number for `id`, e.g. once a new
    /// chapter has been detected. Returns the updated webtoon, or `None` if
    /// `id` isn't followed (it may have been un-followed concurrently).
    pub fn update_chapter(&self, id: u64, new_chapter: u32) -> Option<Webtoon> {
        let mut inner = self.inner.lock().unwrap();
        let webtoon = inner.items.get_mut(&id)?;
        webtoon.current_chapter = new_chapter;
        Some(webtoon.clone())
    }
}

/// Something that can check whether a webtoon has a newer chapter out than
/// the one we last recorded. In a real service, an implementation of this
/// trait would use `reqwest` to call a webtoon-tracking API or scrape a
/// source site's chapter list — there's no public API this project can
/// depend on for that, so every implementation below is simulated instead.
/// `#[async_trait]` (rather than the native `async fn in trait` syntax used
/// by the cache-aside lesson's `Cache` trait) is used here because we
/// actually need `dyn ChapterChecker` — `AppState` below stores one trait
/// object shared between the HTTP handlers and the background job, and
/// native RPITIT async trait methods aren't `dyn`-safe.
#[async_trait::async_trait]
pub trait ChapterChecker: Send + Sync {
    /// Returns `Some(new_chapter_number)` if a chapter newer than
    /// `webtoon.current_chapter` is available, `None` otherwise.
    async fn check(&self, webtoon: &Webtoon) -> Option<u32>;
}

/// Always reports the next chapter number as available. Deterministic and
/// side-effect-free — the "yes, something changed" case for tests.
pub struct AlwaysNewChapterChecker;

#[async_trait::async_trait]
impl ChapterChecker for AlwaysNewChapterChecker {
    async fn check(&self, webtoon: &Webtoon) -> Option<u32> {
        Some(webtoon.current_chapter + 1)
    }
}

/// Never reports a new chapter. Deterministic — the "nothing changed" case
/// for tests.
pub struct NeverNewChapterChecker;

#[async_trait::async_trait]
impl ChapterChecker for NeverNewChapterChecker {
    async fn check(&self, _webtoon: &Webtoon) -> Option<u32> {
        None
    }
}

/// Simulates a realistic "sometimes there's a new chapter, sometimes there
/// isn't" checker: each call has a `probability` chance of reporting the
/// next chapter number. This is what `main.rs` actually runs with, so that
/// running this service produces occasional, not constant, "new chapter"
/// log lines — the deterministic checkers above are for tests, this one is
/// for watching the notifier do something interesting.
pub struct RandomChapterChecker {
    pub probability: f64,
}

#[async_trait::async_trait]
impl ChapterChecker for RandomChapterChecker {
    async fn check(&self, webtoon: &Webtoon) -> Option<u32> {
        if rand::random::<f64>() < self.probability {
            Some(webtoon.current_chapter + 1)
        } else {
            None
        }
    }
}

/// One "a new chapter was found" occurrence, as reported by a single round
/// of [`check_all_webtoons`].
#[derive(Debug, Clone, PartialEq)]
pub struct NewChapterEvent {
    pub id: u64,
    pub title: String,
    pub old_chapter: u32,
    pub new_chapter: u32,
}

/// Runs one round of checking every followed webtoon in `store` against
/// `checker`, updating the store in place and returning every webtoon that
/// got a new chapter this round. This is deliberately a plain `async fn`,
/// separate from the `tokio::spawn` + `tokio::time::interval` loop in
/// [`spawn_notifier`] below — that separation is what makes the interesting
/// logic ("did anything change, and what do we do about it") testable by
/// just calling this function directly, with no timer, no `sleep`, and no
/// real background task involved.
pub async fn check_all_webtoons(
    store: &WebtoonStore,
    checker: &dyn ChapterChecker,
) -> Vec<NewChapterEvent> {
    let mut events = Vec::new();
    for webtoon in store.list() {
        todo!(
            "call checker.check(&webtoon).await; if it returns Some(new_chapter) where \
             new_chapter is strictly greater than webtoon.current_chapter, call \
             store.update_chapter(webtoon.id, new_chapter), push a NewChapterEvent {{ id: \
             webtoon.id, title: webtoon.title.clone(), old_chapter: webtoon.current_chapter, \
             new_chapter }} onto `events`, and tracing::info!(...) that a new chapter was \
             found; a Some(new_chapter) that isn't actually greater (or a None) means do \
             nothing for this webtoon"
        )
    }
    events
}

/// Spawns a background task that calls [`check_all_webtoons`] every
/// `period`, forever, logging each detected new chapter via `tracing`. This
/// is the periodic-polling half of "background jobs," previewed here ahead
/// of Phase 4's proper job-queue lesson
/// (`phase4-backend-advanced/03-background-jobs-and-message-queues`).
/// Given, fully implemented — the interesting part is `check_all_webtoons`,
/// which this just calls on a timer.
pub fn spawn_notifier(
    store: Arc<WebtoonStore>,
    checker: Arc<dyn ChapterChecker>,
    period: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        loop {
            interval.tick().await;
            let events = check_all_webtoons(&store, checker.as_ref()).await;
            for event in &events {
                tracing::info!(
                    webtoon_id = event.id,
                    title = %event.title,
                    old_chapter = event.old_chapter,
                    new_chapter = event.new_chapter,
                    "new chapter detected"
                );
            }
        }
    })
}

/// Shared state for the axum app: the followed-webtoon store, plus the
/// checker used for on-demand `POST /webtoons/{id}/check` calls (the same
/// checker the background job uses, in `main.rs`).
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<WebtoonStore>,
    pub checker: Arc<dyn ChapterChecker>,
}

/// `POST /webtoons` — start following a new webtoon.
pub async fn follow_webtoon(
    State(state): State<AppState>,
    Json(input): Json<FollowWebtoon>,
) -> (StatusCode, Json<Webtoon>) {
    todo!(
        "call state.store.follow(input) to get the newly-followed Webtoon, then return \
         (StatusCode::CREATED, Json(webtoon)) — a POST that creates a resource conventionally \
         answers 201, not 200"
    )
}

/// `GET /webtoons` — list every followed webtoon.
pub async fn list_webtoons(State(state): State<AppState>) -> Json<Vec<Webtoon>> {
    Json(state.store.list())
}

/// The result of an on-demand `POST /webtoons/{id}/check` call.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CheckResult {
    pub webtoon: Webtoon,
    pub new_chapter_found: bool,
}

/// `POST /webtoons/{id}/check` — simulate checking one webtoon right now,
/// outside the background job's schedule (handy for demos: follow
/// something, then check it immediately instead of waiting for the next
/// tick).
pub async fn check_webtoon(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<CheckResult>, WebtoonError> {
    todo!(
        "look up the webtoon with state.store.get(id).ok_or(WebtoonError::NotFound)?; call \
         state.checker.check(&webtoon).await; if it returns Some(new_chapter) that's strictly \
         greater than webtoon.current_chapter, call state.store.update_chapter(id, \
         new_chapter).unwrap() to get the updated Webtoon and return Ok(Json(CheckResult {{ \
         webtoon: updated, new_chapter_found: true }})); otherwise return Ok(Json(CheckResult \
         {{ webtoon, new_chapter_found: false }}))"
    )
}

/// `/webtoons` handles listing and following; `/webtoons/{id}/check`
/// triggers an on-demand check for one webtoon. Same two-route shape as
/// Phase 3's anime catalog CRUD lesson's `app`.
pub fn app(state: AppState) -> Router {
    todo!(
        "Router::new() \
            .route(\"/webtoons\", get(list_webtoons).post(follow_webtoon)) \
            .route(\"/webtoons/{{id}}/check\", post(check_webtoon)) \
            .with_state(state)"
    )
}
