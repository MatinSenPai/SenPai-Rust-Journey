//! Reference solution for 2.6.5 — `RefCell`, `Cell`, and the run-time panic
//! trade.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Tracks two things about a single anime watch-club, through shared (`&self`)
/// methods only.
#[derive(Default)]
pub struct ClubStats {
    members_online: Cell<u32>,
    watch_log: RefCell<Vec<String>>,
}

impl ClubStats {
    /// A fresh `ClubStats`: zero members online, an empty watch log.
    pub fn new() -> Self {
        ClubStats {
            members_online: Cell::new(0),
            watch_log: RefCell::new(Vec::new()),
        }
    }

    /// Records that one more member joined — increases [`Self::members_online`]
    /// by 1.
    pub fn member_joined(&self) {
        self.members_online.set(self.members_online.get() + 1);
    }

    /// The number of members who have joined since this `ClubStats` was
    /// created.
    pub fn members_online(&self) -> u32 {
        self.members_online.get()
    }

    /// Appends `title` to the watch log.
    pub fn log_episode(&self, title: &str) {
        self.watch_log.borrow_mut().push(title.to_string());
    }

    /// Every logged title, in the order [`Self::log_episode`] was called,
    /// oldest first.
    pub fn watch_log(&self) -> Vec<String> {
        self.watch_log.borrow().clone()
    }
}

/// A hype counter with multiple owners, all of whom see and can change the
/// *same* underlying total.
#[derive(Clone, Default)]
pub struct SharedHypeMeter {
    inner: Rc<RefCell<i32>>,
}

impl SharedHypeMeter {
    /// A fresh, single-owner meter starting at 0.
    pub fn new() -> Self {
        SharedHypeMeter {
            inner: Rc::new(RefCell::new(0)),
        }
    }

    /// Adds `amount` to the shared total (negative values lower it).
    pub fn hype_up(&self, amount: i32) {
        *self.inner.borrow_mut() += amount;
    }

    /// The current shared total.
    pub fn hype(&self) -> i32 {
        *self.inner.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_nobody_online_and_no_history() {
        let stats = ClubStats::new();
        assert_eq!(stats.members_online(), 0);
        assert_eq!(stats.watch_log(), Vec::<String>::new());
    }

    #[test]
    fn member_joined_counts_through_a_shared_reference() {
        let stats = ClubStats::new();
        stats.member_joined();
        stats.member_joined();
        stats.member_joined();
        assert_eq!(stats.members_online(), 3);
    }

    #[test]
    fn watch_log_remembers_order() {
        let stats = ClubStats::new();
        stats.log_episode("Frieren");
        stats.log_episode("Bocchi the Rock!");
        assert_eq!(stats.watch_log(), vec!["Frieren", "Bocchi the Rock!"]);
    }

    #[test]
    fn hype_meter_starts_at_zero() {
        let meter = SharedHypeMeter::new();
        assert_eq!(meter.hype(), 0);
    }

    #[test]
    fn hype_up_adds_and_can_go_negative() {
        let meter = SharedHypeMeter::new();
        meter.hype_up(10);
        meter.hype_up(-3);
        assert_eq!(meter.hype(), 7);
    }

    /// `.clone()` on a `SharedHypeMeter` is an `Rc` clone, not a deep copy —
    /// both handles mutate the SAME `RefCell<i32>`.
    #[test]
    fn clones_share_the_same_underlying_total() {
        let original = SharedHypeMeter::new();
        let handle = original.clone();

        original.hype_up(5); // mutate through the first handle
        handle.hype_up(2); // mutate through the second handle

        assert_eq!(original.hype(), 7);
        assert_eq!(handle.hype(), 7);
    }

    /// `RefCell` enforces "shared XOR mutable" at run time instead of compile
    /// time. A second `.borrow_mut()` while the first is still alive violates
    /// that rule, and `RefCell` panics rather than letting two exclusive
    /// borrows coexist.
    #[test]
    #[should_panic(expected = "already borrowed")]
    fn a_second_borrow_mut_panics_while_the_first_is_still_alive() {
        let cell = RefCell::new(0);
        let _first = cell.borrow_mut();
        let _second = cell.borrow_mut();
    }
}
