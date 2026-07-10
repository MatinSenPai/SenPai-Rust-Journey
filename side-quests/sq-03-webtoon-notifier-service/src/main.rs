// Provided for you — no `todo!()`s here, only in `lib.rs`.
use std::sync::Arc;
use std::time::Duration;

use sq_03_webtoon_notifier_service::{
    app, spawn_notifier, AppState, ChapterChecker, RandomChapterChecker, WebtoonStore,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let store = Arc::new(WebtoonStore::default());
    // Real production code would swap this for something that calls a
    // webtoon-tracking API or scrapes a source site via `reqwest`. This one
    // just rolls dice, but it exercises the exact same `ChapterChecker`
    // interface a real implementation would.
    let checker: Arc<dyn ChapterChecker> = Arc::new(RandomChapterChecker { probability: 0.3 });

    // Background job: every 10 seconds, check every followed webtoon and
    // log whenever a (simulated) new chapter shows up.
    let _notifier_handle = spawn_notifier(store.clone(), checker.clone(), Duration::from_secs(10));

    let state = AppState { store, checker };
    let router = app(state);

    let addr = "127.0.0.1:3002";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!(
        "webtoon notifier listening on {addr} — try:\n  \
         curl -X POST http://{addr}/webtoons -H 'content-type: application/json' \
         -d '{{\"title\": \"Solo Leveling\", \"current_chapter\": 179}}'\n  \
         curl http://{addr}/webtoons"
    );

    axum::serve(listener, router).await.expect("server error");
}
