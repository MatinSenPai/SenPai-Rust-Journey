//! Exercises `AnimeStore` directly against a live Postgres — the exact same
//! test names/shape as module 2's in-memory `store_test.rs`, so you can
//! compare the two side by side. Every test here needs `DATABASE_URL` set
//! and cleans out `p3_04_03_anime` first, the same discipline as every
//! other live-Postgres lesson in this repo.

use p3_04_03_anime_catalog_postgres_backed_solution::{
    AnimeError, AnimeStore, CreateAnime, UpdateAnime, WatchStatus,
};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

async fn fresh_store() -> AnimeStore {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run this ignored integration test");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p3_04_03_anime (\
        id BIGSERIAL PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL, rating SMALLINT)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM p3_04_03_anime")
        .execute(&pool)
        .await
        .unwrap();

    AnimeStore::new(pool)
}

fn sample() -> CreateAnime {
    CreateAnime {
        title: "Frieren".to_string(),
        status: WatchStatus::Watching,
        rating: Some(9),
    }
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn create_assigns_an_id_and_persists_the_row() {
    let store = fresh_store().await;
    let anime = store.create(sample()).await.unwrap();
    assert!(anime.id > 0);
    assert_eq!(anime.title, "Frieren");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn create_rejects_an_out_of_range_rating() {
    let store = fresh_store().await;
    let mut input = sample();
    input.rating = Some(11);
    let result = store.create(input).await;
    assert!(matches!(result, Err(AnimeError::InvalidRating(11))));
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn get_finds_a_created_item() {
    let store = fresh_store().await;
    let created = store.create(sample()).await.unwrap();
    let fetched = store.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn get_returns_not_found_for_a_missing_id() {
    let store = fresh_store().await;
    let result = store.get(999_999).await;
    assert!(matches!(result, Err(AnimeError::NotFound)));
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn list_returns_every_item_sorted_by_id() {
    let store = fresh_store().await;
    store.create(sample()).await.unwrap();
    store
        .create(CreateAnime {
            title: "Chainsaw Man".to_string(),
            status: WatchStatus::PlanToWatch,
            rating: None,
        })
        .await
        .unwrap();

    let all = store.list().await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].title, "Frieren");
    assert_eq!(all[1].title, "Chainsaw Man");
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn update_overwrites_only_the_fields_that_were_set() {
    let store = fresh_store().await;
    let created = store.create(sample()).await.unwrap();

    let updated = store
        .update(
            created.id,
            UpdateAnime {
                title: None,
                status: Some(WatchStatus::Completed),
                rating: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.title, "Frieren");
    assert_eq!(updated.status, WatchStatus::Completed);
    assert_eq!(updated.rating, Some(9));
}

#[tokio::test]
#[ignore = "requires a live Postgres reachable at DATABASE_URL"]
#[serial(p3_04_03_anime_db)]
async fn delete_removes_the_item_and_returns_it() {
    let store = fresh_store().await;
    let created = store.create(sample()).await.unwrap();

    let deleted = store.delete(created.id).await.unwrap();
    assert_eq!(deleted, created);
    assert!(matches!(
        store.get(created.id).await,
        Err(AnimeError::NotFound)
    ));
}
