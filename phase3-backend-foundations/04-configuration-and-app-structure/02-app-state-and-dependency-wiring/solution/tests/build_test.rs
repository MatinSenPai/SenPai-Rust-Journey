//! The Build rung: a second dependency (`Notifier`) wired into the state and
//! faked in a test, without touching the first one.

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

use p3_04_02_app_state_and_dependency_wiring_solution::{
    app, AppState, Config, FakeClock, Notifier,
};

#[derive(Default)]
struct Recording(Mutex<Vec<String>>);

impl Notifier for Recording {
    fn notify(&self, message: &str) {
        self.0.lock().unwrap().push(message.to_string());
    }
}

async fn post(router: &axum::Router, title: &str) -> StatusCode {
    let request = Request::post("/watch")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"title":"{title}"}}"#)))
        .unwrap();
    router.clone().oneshot(request).await.unwrap().status()
}

fn fixture() -> (axum::Router, Arc<Recording>) {
    let recording = Arc::new(Recording::default());
    let state = AppState::new(Arc::new(FakeClock::at(0)), Config::default())
        .with_notifier(recording.clone());
    (app(state), recording)
}

#[tokio::test]
async fn each_recorded_watch_notifies_once() {
    let (router, recording) = fixture();
    assert_eq!(post(&router, "Frieren").await, StatusCode::CREATED);
    assert_eq!(post(&router, "  Dandadan ").await, StatusCode::CREATED);
    assert_eq!(
        *recording.0.lock().unwrap(),
        ["watched: Frieren", "watched: Dandadan"]
    );
}

#[tokio::test]
async fn a_rejected_watch_notifies_nobody() {
    let (router, recording) = fixture();
    assert_eq!(post(&router, "   ").await, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(recording.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn the_default_notifier_changes_nothing() {
    let state = AppState::new(Arc::new(FakeClock::at(0)), Config::default());
    let router = app(state);
    assert_eq!(post(&router, "Frieren").await, StatusCode::CREATED);
}
