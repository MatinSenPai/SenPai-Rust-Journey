//! Exercises `NotificationStore` directly against a live Postgres. All
//! tests share the `p5_07_04_notifications` table, so every test carries
//! `#[serial(p5_07_04_notifications_db)]` — see
//! `phase3-backend-foundations/04-postgres-and-sqlx/02-migrations/README.md`
//! for why (a real, reproducible flakiness bug this repo's own test suite
//! hit without it).

use p5_07_04_design_a_notification_system::{CreateNotification, NotificationStore};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn fresh_store() -> NotificationStore {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run this ignored integration test");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p5_07_04_notifications (\
            id BIGSERIAL PRIMARY KEY, \
            user_id TEXT NOT NULL, \
            message TEXT NOT NULL, \
            read BOOLEAN NOT NULL DEFAULT false)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM p5_07_04_notifications")
        .execute(&pool)
        .await
        .unwrap();

    NotificationStore::new(pool)
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn creating_a_notification_persists_it_unread() {
    let store = fresh_store().await;

    let created = store
        .create(CreateNotification {
            user_id: "alice".to_string(),
            message: "your order shipped".to_string(),
        })
        .await
        .unwrap();

    assert!(!created.read);
    let unread = store.list_unread("alice").await.unwrap();
    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0].message, "your order shipped");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn list_unread_only_returns_unread_notifications_for_that_user() {
    let store = fresh_store().await;

    store
        .create(CreateNotification {
            user_id: "alice".to_string(),
            message: "for alice".to_string(),
        })
        .await
        .unwrap();
    let bobs = store
        .create(CreateNotification {
            user_id: "bob".to_string(),
            message: "for bob".to_string(),
        })
        .await
        .unwrap();
    store.mark_read(bobs.id).await.unwrap();

    let alice_unread = store.list_unread("alice").await.unwrap();
    assert_eq!(alice_unread.len(), 1);

    let bob_unread = store.list_unread("bob").await.unwrap();
    assert!(bob_unread.is_empty());
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn marking_a_missing_notification_read_returns_not_found() {
    let store = fresh_store().await;
    let result = store.mark_read(999_999).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn a_subscriber_receives_a_notification_created_after_it_subscribed() {
    let store = fresh_store().await;
    let mut receiver = store.subscribe("alice");

    let created = store
        .create(CreateNotification {
            user_id: "alice".to_string(),
            message: "pushed live".to_string(),
        })
        .await
        .unwrap();

    let pushed = receiver.recv().await.unwrap();
    assert_eq!(pushed.id, created.id);
    assert_eq!(pushed.message, "pushed live");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p5_07_04_notifications_db)]
async fn a_subscriber_for_a_different_user_does_not_see_it() {
    let store = fresh_store().await;
    let mut carol_receiver = store.subscribe("carol");

    store
        .create(CreateNotification {
            user_id: "alice".to_string(),
            message: "not for carol".to_string(),
        })
        .await
        .unwrap();

    assert!(carol_receiver.try_recv().is_err());
}
