use sq_03_webtoon_notifier_service_solution::{
    check_all_webtoons, AlwaysNewChapterChecker, ChapterChecker, FollowWebtoon,
    NeverNewChapterChecker, Webtoon, WebtoonStore,
};

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

    let updated = store.list();
    for webtoon in &updated {
        let expected = match webtoon.title.as_str() {
            "Solo Leveling" => 180,
            "Tower of God" => 601,
            other => panic!("unexpected webtoon in store: {other}"),
        };
        assert_eq!(webtoon.current_chapter, expected);
    }
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
