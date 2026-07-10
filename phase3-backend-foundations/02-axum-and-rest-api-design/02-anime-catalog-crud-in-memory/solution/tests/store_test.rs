//! Exercises `AnimeStore` directly — no `axum`, no HTTP, no
//! `tower::ServiceExt::oneshot`. Same "test the pure logic on its own"
//! discipline as `run_echo` in module 1.

use p3_02_02_anime_catalog_crud_in_memory_solution::{
    AnimeError, AnimeStore, CreateAnime, UpdateAnime, WatchStatus,
};

fn sample() -> CreateAnime {
    CreateAnime {
        title: "Frieren".to_string(),
        status: WatchStatus::Watching,
        rating: Some(9),
    }
}

#[test]
fn create_assigns_an_id_starting_at_one() {
    let store = AnimeStore::default();
    let anime = store.create(sample()).unwrap();
    assert_eq!(anime.id, 1);
    assert_eq!(anime.title, "Frieren");
}

#[test]
fn create_rejects_an_out_of_range_rating() {
    let store = AnimeStore::default();
    let mut input = sample();
    input.rating = Some(11);
    assert_eq!(store.create(input), Err(AnimeError::InvalidRating(11)));
}

#[test]
fn create_rejects_a_zero_rating() {
    let store = AnimeStore::default();
    let mut input = sample();
    input.rating = Some(0);
    assert_eq!(store.create(input), Err(AnimeError::InvalidRating(0)));
}

#[test]
fn get_finds_a_created_item() {
    let store = AnimeStore::default();
    let created = store.create(sample()).unwrap();
    let fetched = store.get(created.id).unwrap();
    assert_eq!(fetched, created);
}

#[test]
fn get_returns_not_found_for_a_missing_id() {
    let store = AnimeStore::default();
    assert_eq!(store.get(999), Err(AnimeError::NotFound));
}

#[test]
fn list_is_empty_for_a_fresh_store() {
    let store = AnimeStore::default();
    assert_eq!(store.list(), vec![]);
}

#[test]
fn list_returns_every_item_sorted_by_id() {
    let store = AnimeStore::default();
    store.create(sample()).unwrap();
    store
        .create(CreateAnime {
            title: "Chainsaw Man".to_string(),
            status: WatchStatus::PlanToWatch,
            rating: None,
        })
        .unwrap();

    let all = store.list();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].title, "Frieren");
    assert_eq!(all[1].title, "Chainsaw Man");
}

#[test]
fn update_overwrites_only_the_fields_that_were_set() {
    let store = AnimeStore::default();
    let created = store.create(sample()).unwrap();

    let updated = store
        .update(
            created.id,
            UpdateAnime {
                title: None,
                status: Some(WatchStatus::Completed),
                rating: None,
            },
        )
        .unwrap();

    assert_eq!(updated.title, "Frieren"); // untouched
    assert_eq!(updated.status, WatchStatus::Completed); // changed
    assert_eq!(updated.rating, Some(9)); // untouched
}

#[test]
fn update_rejects_an_out_of_range_rating() {
    let store = AnimeStore::default();
    let created = store.create(sample()).unwrap();
    let result = store.update(
        created.id,
        UpdateAnime {
            rating: Some(20),
            ..Default::default()
        },
    );
    assert_eq!(result, Err(AnimeError::InvalidRating(20)));
}

#[test]
fn update_returns_not_found_for_a_missing_id() {
    let store = AnimeStore::default();
    let result = store.update(999, UpdateAnime::default());
    assert_eq!(result, Err(AnimeError::NotFound));
}

#[test]
fn delete_removes_the_item_and_returns_it() {
    let store = AnimeStore::default();
    let created = store.create(sample()).unwrap();

    let deleted = store.delete(created.id).unwrap();
    assert_eq!(deleted, created);
    assert_eq!(store.get(created.id), Err(AnimeError::NotFound));
}

#[test]
fn delete_returns_not_found_for_a_missing_id() {
    let store = AnimeStore::default();
    assert_eq!(store.delete(999), Err(AnimeError::NotFound));
}
