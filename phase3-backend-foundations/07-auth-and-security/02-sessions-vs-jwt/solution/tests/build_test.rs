//! The Build and Challenge rungs: sliding expiry and "log out other devices".

use std::sync::Arc;

use p3_07_02_sessions_vs_jwt_solution::{ManualClock, SessionStore};

fn fixture() -> (Arc<ManualClock>, SessionStore) {
    let clock = Arc::new(ManualClock::new(1_000));
    (clock.clone(), SessionStore::new(clock, 100))
}

#[test]
fn touch_pushes_expiry_a_full_lifetime_from_now() {
    let (clock, store) = fixture();
    let id = store.create("matin"); // expires at 1_100
    clock.set(1_090);
    assert!(store.touch(&id)); // now expires at 1_190
    clock.set(1_150);
    assert_eq!(store.lookup(&id), Some("matin".to_string()));
    clock.set(1_190);
    assert_eq!(store.lookup(&id), None);
}

#[test]
fn touch_does_not_resurrect_an_expired_or_unknown_session() {
    let (clock, store) = fixture();
    let id = store.create("matin");
    clock.set(1_100);
    assert!(!store.touch(&id));
    assert_eq!(store.lookup(&id), None);
    assert!(!store.touch("nope"));
}

#[test]
fn revoke_others_keeps_only_the_named_session() {
    let (_, store) = fixture();
    let keep = store.create("matin");
    let phone = store.create("matin");
    let tablet = store.create("matin");
    let sara = store.create("sara");
    assert_eq!(store.revoke_others("matin", &keep), 2);
    assert_eq!(store.lookup(&keep), Some("matin".to_string()));
    assert_eq!(store.lookup(&phone), None);
    assert_eq!(store.lookup(&tablet), None);
    assert_eq!(store.lookup(&sara), Some("sara".to_string()));
}
