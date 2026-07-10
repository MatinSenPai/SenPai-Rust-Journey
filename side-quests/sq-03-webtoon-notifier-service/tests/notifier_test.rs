//! Exercises `check_all_webtoons` — the "did anything change" logic the
//! background job runs on a timer — as a plain function call. No
//! `tokio::time::interval`, no `sleep`, no real background task: this is
//! exactly why `check_all_webtoons` is factored out separately from
//! `spawn_notifier` in `src/lib.rs`.

use sq_03_webtoon_notifier_service::{
    check_all_webtoons, AlwaysNewChapterChecker, ChapterChecker, FollowWebtoon,
    NeverNewChapterChecker, Webtoon, WebtoonStore,
};

/// A checker that reports the *same* chapter number as current — used to
/// prove `check_all_webtoons` only counts a check as "new" when the
/// reported chapter is strictly greater than what's already stored, not
/// merely `Some(_)`.
struct SameChapterChecker;

#[async_trait::async_trait]
impl ChapterChecker for SameChapterChecker {
    async fn check(&self, webtoon: &Webtoon) -> Option<u32> {
        Some(webtoon.current_chapter)
    }
}

#[tokio::test]
async fn detects_and_records_a_new_chapter_for_every_followed_webtoon() {
    let store = WebtoonStore::default();
    store.follow(FollowWebtoon {
        title: "Solo Leveling".to_string(),
        current_chapter: 179,
    });
    store.follow(FollowWebtoon {
        title: "Tower of God".to_string(),
        current_chapter: 600,
    });

    let events = check_all_webtoons(&store, &AlwaysNewChapterChecker).await;

    assert_eq!(events.len(), 2);
    for event in &events {
        assert_eq!(event.new_chapter, event.old_chapter + 1);
    }

    // The store itself should have been updated, not just the returned events.
    let updated = store.list();
    assert!(updated.iter().all(|w| w.current_chapter
        == match w.title.as_str() {
            "Solo Leveling" => 180,
            "Tower of God" => 601,
            other => panic!("unexpected webtoon in store: {other}"),
        }));
}

#[tokio::test]
async fn reports_no_events_when_nothing_is_new() {
    let store = WebtoonStore::default();
    store.follow(FollowWebtoon {
        title: "Omniscient Reader".to_string(),
        current_chapter: 42,
    });

    let events = check_all_webtoons(&store, &NeverNewChapterChecker).await;

    assert!(events.is_empty());
    assert_eq!(
        store.list()[0].current_chapter,
        42,
        "store must be untouched"
    );
}

#[tokio::test]
async fn a_reported_chapter_that_is_not_strictly_greater_does_not_count_as_new() {
    let store = WebtoonStore::default();
    store.follow(FollowWebtoon {
        title: "Lookism".to_string(),
        current_chapter: 500,
    });

    let events = check_all_webtoons(&store, &SameChapterChecker).await;

    assert!(
        events.is_empty(),
        "a checker reporting the same chapter number should not be treated as a new release"
    );
    assert_eq!(store.list()[0].current_chapter, 500);
}

#[tokio::test]
async fn an_empty_store_produces_no_events() {
    let store = WebtoonStore::default();
    let events = check_all_webtoons(&store, &AlwaysNewChapterChecker).await;
    assert!(events.is_empty());
}
