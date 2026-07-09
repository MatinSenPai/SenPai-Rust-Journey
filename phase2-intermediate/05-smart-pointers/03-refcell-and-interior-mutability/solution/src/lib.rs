use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SharedCounter {
    inner: Rc<RefCell<i32>>,
}

impl SharedCounter {
    pub fn new() -> Self {
        SharedCounter {
            inner: Rc::new(RefCell::new(0)),
        }
    }

    pub fn increment(&self) {
        *self.inner.borrow_mut() += 1;
    }

    pub fn get(&self) -> i32 {
        *self.inner.borrow()
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
        let handle = original.clone();

        original.increment();
        handle.increment();
        original.increment();

        assert_eq!(original.get(), 3);
        assert_eq!(handle.get(), 3);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn a_second_borrow_mut_panics_while_the_first_is_still_alive() {
        let cell = RefCell::new(0);
        let _first = cell.borrow_mut();
        let _second = cell.borrow_mut();
    }
}
