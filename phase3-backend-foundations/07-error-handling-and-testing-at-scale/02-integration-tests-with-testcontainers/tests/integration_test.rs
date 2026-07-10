//! Real integration tests: each one boots an actual, ephemeral,
//! throwaway Postgres via `testcontainers`, connects a real `sqlx::PgPool`
//! to it, and exercises `WidgetRepository` against real SQL — no
//! in-memory test double stands in for Postgres here at all. Every test in
//! this file is `#[ignore]`-gated: it needs a running Docker daemon, which
//! this repo's sandbox does not have (Docker CLI is present, but no daemon
//! is running). Run these where Docker *is* available with:
//!
//! ```sh
//! cargo test -p p3-07-02-integration-tests-with-testcontainers -- --ignored
//! ```

use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

use p3_07_02_integration_tests_with_testcontainers::{run_schema, WidgetRepository};

/// Boots a fresh, empty Postgres container and returns a connected pool
/// plus the container handle. The container **must** stay alive for as
/// long as the pool is used — it's dropped (and torn down) automatically
/// when the caller's `_container` binding goes out of scope, so every test
/// below binds it with a leading underscore rather than discarding it, to
/// keep it alive for the whole test.
async fn live_postgres() -> (
    PgPool,
    testcontainers_modules::testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default()
        .start()
        .await
        .expect("starting the Postgres container should succeed with a running Docker daemon");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let pool = PgPool::connect(&url)
        .await
        .expect("connecting to the just-started container should succeed");
    run_schema(&pool).await.unwrap();

    (pool, container)
}

#[tokio::test]
#[ignore = "requires a running Docker daemon (testcontainers boots a real Postgres container)"]
async fn creating_a_widget_assigns_an_id() {
    let (pool, _container) = live_postgres().await;
    let repo = WidgetRepository::new(pool);

    let widget = repo.create("Bolt", 100).await.unwrap();

    assert!(widget.id > 0);
    assert_eq!(widget.name, "Bolt");
    assert_eq!(widget.quantity, 100);
}

#[tokio::test]
#[ignore = "requires a running Docker daemon (testcontainers boots a real Postgres container)"]
async fn getting_a_created_widget_finds_it() {
    let (pool, _container) = live_postgres().await;
    let repo = WidgetRepository::new(pool);

    let created = repo.create("Washer", 250).await.unwrap();
    let fetched = repo.get(created.id).await.unwrap();

    assert_eq!(fetched, Some(created));
}

#[tokio::test]
#[ignore = "requires a running Docker daemon (testcontainers boots a real Postgres container)"]
async fn getting_a_missing_widget_returns_none() {
    let (pool, _container) = live_postgres().await;
    let repo = WidgetRepository::new(pool);

    assert_eq!(repo.get(999).await.unwrap(), None);
}

#[tokio::test]
#[ignore = "requires a running Docker daemon (testcontainers boots a real Postgres container)"]
async fn listing_returns_every_widget_in_this_container_ordered_by_id() {
    let (pool, _container) = live_postgres().await;
    let repo = WidgetRepository::new(pool);

    // This container is freshly created for this test alone, so there's no
    // leftover state from any other test or lesson to account for here —
    // unlike the shared `taskforge` database other lessons in this repo
    // reuse, a fresh container starts genuinely empty every time.
    repo.create("Bolt", 100).await.unwrap();
    repo.create("Washer", 250).await.unwrap();

    let widgets = repo.list().await.unwrap();

    assert_eq!(widgets.len(), 2);
    assert_eq!(widgets[0].name, "Bolt");
    assert_eq!(widgets[1].name, "Washer");
}
