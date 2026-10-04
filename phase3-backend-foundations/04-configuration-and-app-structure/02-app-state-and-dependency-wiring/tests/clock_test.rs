//! The two clocks, checked on their own: no HTTP, no state.

use std::sync::Arc;

use p3_04_02_app_state_and_dependency_wiring::{Clock, FakeClock, SystemClock};

#[test]
fn a_fake_clock_reads_the_time_it_was_given() {
    assert_eq!(FakeClock::at(1_000).now(), 1_000);
}

#[test]
fn a_fake_clock_does_not_move_by_itself() {
    let clock = FakeClock::at(5);
    assert_eq!(clock.now(), 5);
    assert_eq!(clock.now(), 5);
}

#[test]
fn advance_adds_to_the_current_time() {
    let clock = FakeClock::at(100);
    clock.advance(30);
    clock.advance(0);
    clock.advance(5);
    assert_eq!(clock.now(), 135);
}

#[test]
fn a_fake_clock_can_be_used_as_a_trait_object() {
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::at(42));
    assert_eq!(clock.now(), 42);
}

#[test]
fn every_holder_of_the_same_fake_clock_sees_the_same_time() {
    let clock = Arc::new(FakeClock::at(10));
    let seen_by_the_app: Arc<dyn Clock> = clock.clone();
    clock.advance(7);
    assert_eq!(seen_by_the_app.now(), 17);
}

#[test]
fn the_system_clock_is_after_2020_and_never_goes_backwards_in_a_test() {
    let first = SystemClock.now();
    let second = SystemClock.now();
    // 2020-01-01T00:00:00Z
    assert!(first > 1_577_836_800, "got {first}");
    assert!(second >= first);
}
