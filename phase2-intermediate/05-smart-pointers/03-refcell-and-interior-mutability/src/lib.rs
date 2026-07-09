use std::cell::RefCell;
use std::rc::Rc;

/// A counter that can have multiple owners (via `.clone()`), all of whom
/// see and can mutate the *same* underlying count.
///
/// Compare this to `#[derive(Clone)]` on a struct with a plain `i32` field:
/// cloning that would deep-copy the integer, giving you two independent
/// counters. Here, `inner` is `Rc<RefCell<i32>>` — cloning a `SharedCounter`
/// clones the `Rc` (cheap, see last lesson), which means every clone still
/// points at the exact same `RefCell<i32>` on the heap.
#[derive(Clone, Default)]
pub struct SharedCounter {
    inner: Rc<RefCell<i32>>,
}

impl SharedCounter {
    /// Creates a new counter, starting at 0, with a single owner.
    pub fn new() -> Self {
        todo!("SharedCounter {{ inner: Rc::new(RefCell::new(0)) }}")
    }

    /// Increments the counter by 1.
    ///
    /// Note the signature: `&self`, not `&mut self`. This is the entire
    /// point of interior mutability — `RefCell::borrow_mut()` gives us
    /// exclusive, mutable access to the `i32` *through* a plain shared
    /// reference, with the "only one mutable borrow at a time" rule
    /// enforced at run time by the `RefCell` instead of at compile time by
    /// the borrow checker.
    pub fn increment(&self) {
        todo!("*self.inner.borrow_mut() += 1")
    }

    /// Returns the current count.
    pub fn get(&self) -> i32 {
        todo!("*self.inner.borrow()")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        let counter = SharedCounter::new();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn increments() {
        let counter = SharedCounter::new();
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn clones_share_the_same_underlying_counter() {
        let original = SharedCounter::new();
        let handle = original.clone(); // a second owner of the SAME RefCell<i32>

        original.increment(); // mutate through the first handle
        handle.increment(); // mutate through the second handle
        original.increment(); // and the first again

        // Both handles observe every increment, from either side, because
        // they're two `Rc` pointers into one shared `RefCell<i32>` — not
        // two independent counters.
        assert_eq!(original.get(), 3);
        assert_eq!(handle.get(), 3);
    }

    /// `RefCell` enforces "shared XOR mutable" at run time instead of
    /// compile time. Taking a second `.borrow_mut()` while the first is
    /// still alive (not yet dropped) violates that rule, and `RefCell`
    /// panics rather than silently letting two exclusive borrows coexist.
    #[test]
    #[should_panic(expected = "already borrowed")]
    fn a_second_borrow_mut_panics_while_the_first_is_still_alive() {
        let cell = RefCell::new(0);
        let _first = cell.borrow_mut();
        let _second = cell.borrow_mut(); // panics: already borrowed: BorrowMutError
    }
}
