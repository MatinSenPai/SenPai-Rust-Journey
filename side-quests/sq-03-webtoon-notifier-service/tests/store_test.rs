//! Exercises `WebtoonStore` through the crate's public surface — same
//! pattern as Phase 3's anime catalog CRUD lesson's `store_test.rs`.

use sq_03_webtoon_notifier_service::{FollowWebtoon, WebtoonStore};

#[test]
fn following_a_webtoon_assigns_an_id_and_stores_it() {
    let store = WebtoonStore::default();
    let webtoon = store.follow(FollowWebtoon {
        title: "Solo Leveling".to_string(),
        current_chapter: 100,
    });

    assert_eq!(webtoon.title, "Solo Leveling");
    assert_eq!(webtoon.current_chapter, 100);
    assert_eq!(store.get(webtoon.id), Some(webtoon));
}

#[test]
fn each_followed_webtoon_gets_a_distinct_id() {
    let store = WebtoonStore::default();
    let a = store.follow(FollowWebtoon {
        title: "Tower of God".to_string(),
        current_chapter: 5,
    });
    let b = store.follow(FollowWebtoon {
        title: "The Beginning After the End".to_string(),
        current_chapter: 20,
    });

    assert_ne!(a.id, b.id);
}

#[test]
fn list_is_sorted_by_id_and_reflects_every_followed_webtoon() {
    let store = WebtoonStore::default();
    store.follow(FollowWebtoon {
        title: "Tower of God".to_string(),
        current_chapter: 5,
    });
    store.follow(FollowWebtoon {
        title: "The Beginning After the End".to_string(),
        current_chapter: 20,
    });

    let list = store.list();
    assert_eq!(list.len(), 2);
    assert!(list[0].id < list[1].id);
}

#[test]
fn list_starts_empty() {
    let store = WebtoonStore::default();
    assert!(store.list().is_empty());
}

#[test]
fn get_returns_none_for_an_unfollowed_id() {
    let store = WebtoonStore::default();
    assert_eq!(store.get(9999), None);
}

#[test]
fn update_chapter_updates_an_existing_webtoon() {
    let store = WebtoonStore::default();
    let webtoon = store.follow(FollowWebtoon {
        title: "Omniscient Reader".to_string(),
        current_chapter: 10,
    });

    let updated = store.update_chapter(webtoon.id, 11).unwrap();
    assert_eq!(updated.current_chapter, 11);
    assert_eq!(store.get(webtoon.id).unwrap().current_chapter, 11);
}

#[test]
fn update_chapter_returns_none_for_an_unfollowed_id() {
    let store = WebtoonStore::default();
    assert_eq!(store.update_chapter(9999, 5), None);
}
