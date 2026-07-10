//! Reference solution — see `SOLUTION.md` for the reasoning behind each
//! filled-in `todo!()`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Webtoon {
    pub id: u64,
    pub title: String,
    pub current_chapter: u32,
}

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

#[derive(Default)]
pub struct WebtoonStore {
    inner: Mutex<StoreInner>,
}

impl WebtoonStore {
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

    pub fn get(&self, id: u64) -> Option<Webtoon> {
        self.inner.lock().unwrap().items.get(&id).cloned()
    }

    pub fn list(&self) -> Vec<Webtoon> {
        let inner = self.inner.lock().unwrap();
        let mut items: Vec<Webtoon> = inner.items.values().cloned().collect();
        items.sort_by_key(|w| w.id);
        items
    }

    pub fn update_chapter(&self, id: u64, new_chapter: u32) -> Option<Webtoon> {
        let mut inner = self.inner.lock().unwrap();
        let webtoon = inner.items.get_mut(&id)?;
        webtoon.current_chapter = new_chapter;
        Some(webtoon.clone())
    }
}

#[async_trait::async_trait]
pub trait ChapterChecker: Send + Sync {
    async fn check(&self, webtoon: &Webtoon) -> Option<u32>;
}

pub struct AlwaysNewChapterChecker;

#[async_trait::async_trait]
impl ChapterChecker for AlwaysNewChapterChecker {
    async fn check(&self, webtoon: &Webtoon) -> Option<u32> {
        Some(webtoon.current_chapter + 1)
    }
}

pub struct NeverNewChapterChecker;

#[async_trait::async_trait]
impl ChapterChecker for NeverNewChapterChecker {
    async fn check(&self, _webtoon: &Webtoon) -> Option<u32> {
        None
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct NewChapterEvent {
    pub id: u64,
    pub title: String,
    pub old_chapter: u32,
    pub new_chapter: u32,
}

pub async fn check_all_webtoons(
    store: &WebtoonStore,
    checker: &dyn ChapterChecker,
) -> Vec<NewChapterEvent> {
    let mut events = Vec::new();
    for webtoon in store.list() {
        if let Some(new_chapter) = checker.check(&webtoon).await {
            if new_chapter > webtoon.current_chapter
                && store.update_chapter(webtoon.id, new_chapter).is_some()
            {
                tracing::info!(
                    webtoon_id = webtoon.id,
                    title = %webtoon.title,
                    old_chapter = webtoon.current_chapter,
                    new_chapter,
                    "new chapter detected"
                );
                events.push(NewChapterEvent {
                    id: webtoon.id,
                    title: webtoon.title.clone(),
                    old_chapter: webtoon.current_chapter,
                    new_chapter,
                });
            }
        }
    }
    events
}

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

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<WebtoonStore>,
    pub checker: Arc<dyn ChapterChecker>,
}

pub async fn follow_webtoon(
    State(state): State<AppState>,
    Json(input): Json<FollowWebtoon>,
) -> (StatusCode, Json<Webtoon>) {
    let webtoon = state.store.follow(input);
    (StatusCode::CREATED, Json(webtoon))
}

pub async fn list_webtoons(State(state): State<AppState>) -> Json<Vec<Webtoon>> {
    Json(state.store.list())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CheckResult {
    pub webtoon: Webtoon,
    pub new_chapter_found: bool,
}

pub async fn check_webtoon(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<CheckResult>, WebtoonError> {
    let webtoon = state.store.get(id).ok_or(WebtoonError::NotFound)?;
    let found = state.checker.check(&webtoon).await;

    match found {
        Some(new_chapter) if new_chapter > webtoon.current_chapter => {
            let updated = state
                .store
                .update_chapter(id, new_chapter)
                .expect("just fetched this webtoon, it must still exist");
            Ok(Json(CheckResult {
                webtoon: updated,
                new_chapter_found: true,
            }))
        }
        _ => Ok(Json(CheckResult {
            webtoon,
            new_chapter_found: false,
        })),
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/webtoons", get(list_webtoons).post(follow_webtoon))
        .route("/webtoons/{id}/check", post(check_webtoon))
        .with_state(state)
}
