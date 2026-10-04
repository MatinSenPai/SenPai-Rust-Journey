//! The in-memory store, checked with plain calls: no HTTP, no clock.

use p3_04_02_app_state_and_dependency_wiring_solution::{Watch, WatchStore};

#[test]
fn ids_start_at_one_and_count_up() {
    let store = WatchStore::default();
    assert_eq!(store.add("Frieren", 10).id, 1);
    assert_eq!(store.add("Dandadan", 20).id, 2);
    assert_eq!(store.add("Frieren", 30).id, 3);
}

#[test]
fn add_returns_the_entry_it_stored() {
    let store = WatchStore::default();
    let watch = store.add("Frieren", 1_700_000_000);
    assert_eq!(
        watch,
        Watch {
            id: 1,
            title: "Frieren".to_string(),
            watched_at: 1_700_000_000
        }
    );
}

#[test]
fn the_title_is_stored_exactly_as_given() {
    let store = WatchStore::default();
    assert_eq!(store.add("  spaced  ", 1).title, "  spaced  ");
}

#[test]
fn count_counts_every_entry() {
    let store = WatchStore::default();
    assert_eq!(store.count(), 0);
    store.add("a", 1);
    store.add("b", 2);
    assert_eq!(store.count(), 2);
}

#[test]
fn since_keeps_entries_at_or_after_the_cutoff() {
    let store = WatchStore::default();
    store.add("old", 100);
    store.add("edge", 200);
    store.add("new", 300);
    let titles: Vec<String> = store.since(200).into_iter().map(|w| w.title).collect();
    assert_eq!(titles, ["edge", "new"]);
}

#[test]
fn since_zero_returns_everything() {
    let store = WatchStore::default();
    store.add("a", 0);
    store.add("b", 5);
    assert_eq!(store.since(0).len(), 2);
}

#[test]
fn since_keeps_the_order_entries_were_added_in() {
    let store = WatchStore::default();
    store.add("later-time-first", 500);
    store.add("earlier-time-second", 400);
    let titles: Vec<String> = store.since(0).into_iter().map(|w| w.title).collect();
    assert_eq!(titles, ["later-time-first", "earlier-time-second"]);
}

#[test]
fn since_a_future_cutoff_is_empty() {
    let store = WatchStore::default();
    store.add("a", 10);
    assert!(store.since(11).is_empty());
}
