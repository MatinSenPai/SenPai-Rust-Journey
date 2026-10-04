//! `SessionStore` with a clock the test moves by hand. No HTTP here.

use std::sync::Arc;

use p3_07_02_sessions_vs_jwt::{ManualClock, SessionStore};

const TTL: u64 = 100;

fn fixture() -> (Arc<ManualClock>, SessionStore) {
    let clock = Arc::new(ManualClock::new(1_000));
    let store = SessionStore::new(clock.clone(), TTL);
    (clock, store)
}

#[test]
fn create_returns_a_32_char_lowercase_hex_id() {
    let (_, store) = fixture();
    let id = store.create("matin");
    assert_eq!(id.len(), 32);
    assert!(id
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
}

#[test]
fn every_create_gets_a_different_id_even_for_the_same_user() {
    let (_, store) = fixture();
    assert_ne!(store.create("matin"), store.create("matin"));
    assert_eq!(store.len(), 2);
}

#[test]
fn lookup_returns_the_user_of_a_live_session() {
    let (_, store) = fixture();
    let id = store.create("matin");
    assert_eq!(store.lookup(&id), Some("matin".to_string()));
}

#[test]
fn lookup_of_an_unknown_id_is_none() {
    let (_, store) = fixture();
    assert_eq!(store.lookup("nope"), None);
}

#[test]
fn a_session_is_live_until_the_instant_it_expires() {
    let (clock, store) = fixture();
    let id = store.create("matin"); // expires_at = 1_100
    clock.set(1_099);
    assert_eq!(store.lookup(&id), Some("matin".to_string()));
    clock.set(1_100);
    assert_eq!(store.lookup(&id), None);
}

#[test]
fn looking_up_an_expired_session_removes_it_from_the_store() {
    let (clock, store) = fixture();
    let id = store.create("matin");
    clock.advance(TTL + 5);
    assert_eq!(store.len(), 1, "nothing purges it until someone looks");
    assert_eq!(store.lookup(&id), None);
    assert_eq!(store.len(), 0);
}

#[test]
fn revoke_kills_a_live_session_and_says_so() {
    let (_, store) = fixture();
    let id = store.create("matin");
    assert!(store.revoke(&id));
    assert_eq!(store.lookup(&id), None);
}

#[test]
fn revoking_twice_or_revoking_an_unknown_id_returns_false() {
    let (_, store) = fixture();
    let id = store.create("matin");
    assert!(store.revoke(&id));
    assert!(!store.revoke(&id));
    assert!(!store.revoke("nope"));
}

#[test]
fn revoking_an_expired_session_returns_false_but_still_removes_it() {
    let (clock, store) = fixture();
    let id = store.create("matin");
    clock.advance(TTL);
    assert!(!store.revoke(&id));
    assert_eq!(store.len(), 0);
}

#[test]
fn revoke_all_for_removes_every_live_session_of_that_user_only() {
    let (_, store) = fixture();
    let a = store.create("matin");
    let b = store.create("matin");
    let other = store.create("sara");
    assert_eq!(store.revoke_all_for("matin"), 2);
    assert_eq!(store.lookup(&a), None);
    assert_eq!(store.lookup(&b), None);
    assert_eq!(store.lookup(&other), Some("sara".to_string()));
}

#[test]
fn revoke_all_for_counts_only_sessions_that_were_still_live() {
    let (clock, store) = fixture();
    store.create("matin"); // will be expired
    clock.advance(60);
    store.create("matin"); // expires at 1_160
    clock.advance(40); // now 1_100: first expired, second live
    assert_eq!(store.revoke_all_for("matin"), 1);
    assert_eq!(store.len(), 0);
}

#[test]
fn revoke_all_for_a_user_with_no_sessions_is_zero() {
    let (_, store) = fixture();
    assert_eq!(store.revoke_all_for("ghost"), 0);
}
