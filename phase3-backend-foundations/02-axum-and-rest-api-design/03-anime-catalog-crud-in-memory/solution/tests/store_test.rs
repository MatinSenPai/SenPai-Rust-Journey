//! Exercises `AnimeStore` directly: no `axum`, no HTTP, no `oneshot`, no
//! runtime. The store is plain Rust, so its tests are plain `#[test]`s.

use p3_02_03_anime_catalog_crud_in_memory_solution::{
    AnimeError, AnimeStore, CreateAnime, UpdateAnime, WatchStatus,
};

fn frieren() -> CreateAnime {
    CreateAnime {
        title: "Frieren".to_string(),
        status: WatchStatus::Watching,
        rating: Some(9),
    }
}

fn titled(title: &str) -> CreateAnime {
    CreateAnime {
        title: title.to_string(),
        status: WatchStatus::PlanToWatch,
        rating: None,
    }
}

#[test]
fn create_assigns_ids_starting_at_one_and_counting_up() {
    let store = AnimeStore::default();
    let first = store.create(frieren()).unwrap();
    let second = store.create(titled("Chainsaw Man")).unwrap();
    assert_eq!(first.id, 1);
    assert_eq!(first.title, "Frieren");
    assert_eq!(second.id, 2);
}

#[test]
fn create_rejects_an_out_of_range_rating() {
    let store = AnimeStore::default();
    let mut input = frieren();
    input.rating = Some(11);
    assert_eq!(store.create(input), Err(AnimeError::InvalidRating(11)));
}

#[test]
fn create_rejects_a_zero_rating() {
    let store = AnimeStore::default();
    let mut input = frieren();
    input.rating = Some(0);
    assert_eq!(store.create(input), Err(AnimeError::InvalidRating(0)));
}

#[test]
fn a_rejected_create_does_not_use_up_an_id() {
    let store = AnimeStore::default();
    let mut bad = frieren();
    bad.rating = Some(11);
    assert!(store.create(bad).is_err());
    assert_eq!(store.create(frieren()).unwrap().id, 1);
    assert_eq!(store.list().len(), 1);
}

#[test]
fn get_finds_a_created_item() {
    let store = AnimeStore::default();
    let created = store.create(frieren()).unwrap();
    assert_eq!(store.get(created.id), Ok(created));
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
    // Twenty items: a `HashMap` would almost never hand these back in order
    // by accident, so an unsorted `list` fails here.
    let store = AnimeStore::default();
    for n in 1..=20 {
        store.create(titled(&format!("show {n}"))).unwrap();
    }
    let ids: Vec<u64> = store.list().into_iter().map(|anime| anime.id).collect();
    assert_eq!(ids, (1..=20).collect::<Vec<u64>>());
}

#[test]
fn update_overwrites_only_the_fields_that_were_set() {
    let store = AnimeStore::default();
    let created = store.create(frieren()).unwrap();
    let change = UpdateAnime {
        status: Some(WatchStatus::Completed),
        ..Default::default()
    };

    let updated = store.update(created.id, change).unwrap();

    assert_eq!(updated.title, "Frieren"); // untouched
    assert_eq!(updated.status, WatchStatus::Completed); // changed
    assert_eq!(updated.rating, Some(9)); // untouched
    assert_eq!(store.get(created.id), Ok(updated)); // and it was stored
}

#[test]
fn update_rejects_an_out_of_range_rating_and_changes_nothing() {
    let store = AnimeStore::default();
    let created = store.create(frieren()).unwrap();
    let change = UpdateAnime {
        title: Some("renamed".to_string()),
        rating: Some(20),
        ..Default::default()
    };
    assert_eq!(
        store.update(created.id, change),
        Err(AnimeError::InvalidRating(20))
    );
    assert_eq!(store.get(created.id), Ok(created));
}

#[test]
fn update_checks_the_rating_before_the_id() {
    let store = AnimeStore::default();
    let change = UpdateAnime {
        rating: Some(20),
        ..Default::default()
    };
    assert_eq!(
        store.update(999, change),
        Err(AnimeError::InvalidRating(20))
    );
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
    let created = store.create(frieren()).unwrap();

    assert_eq!(store.delete(created.id), Ok(created.clone()));
    assert_eq!(store.get(created.id), Err(AnimeError::NotFound));
}

#[test]
fn delete_returns_not_found_for_a_missing_id() {
    let store = AnimeStore::default();
    assert_eq!(store.delete(999), Err(AnimeError::NotFound));
}

#[test]
fn ids_are_never_reused_after_a_delete() {
    let store = AnimeStore::default();
    let first = store.create(frieren()).unwrap();
    store.delete(first.id).unwrap();
    assert_eq!(store.create(titled("Dandadan")).unwrap().id, 2);
}
