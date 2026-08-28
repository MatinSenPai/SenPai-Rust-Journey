//! Reference solution for 1.2.5 — `Drop` and RAII.
//!
//! `Tracker` records its own cleanup in a thread-local log, so the order the
//! compiler drops things in is something a test can assert on.

use std::cell::RefCell;

thread_local! {
    static DROP_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn record(name: &str) {
    DROP_LOG.with(|log| log.borrow_mut().push(name.to_string()));
}

/// Every name cleaned up since the last time the log was taken, oldest first.
///
/// Taking it also empties it, so a function that ends with `take_log()` hands
/// back exactly what it cleaned up and leaves nothing behind for the next one.
pub fn take_log() -> Vec<String> {
    DROP_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

/// A value that announces its own cleanup by recording `name` in the log.
pub struct Tracker {
    name: String,
}

impl Tracker {
    pub fn new(name: &str) -> Tracker {
        Tracker {
            name: name.to_string(),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        record(&self.name);
    }
}

/// Takes ownership of a `Tracker` and does nothing else with it.
pub fn consume(_tracker: Tracker) {}

/// The order three values declared in one block are cleaned up in.
///
/// # Examples
///
/// `scope_order()` returns `["c", "b", "a"]`.
pub fn scope_order() -> Vec<String> {
    {
        let _a = Tracker::new("a");
        let _b = Tracker::new("b");
        let _c = Tracker::new("c");
    }
    take_log()
}

/// Two values in one block, the first of them let go before the second.
///
/// # Examples
///
/// `release_early()` returns `["early", "late"]`.
pub fn release_early() -> Vec<String> {
    {
        let early = Tracker::new("early");
        let _late = Tracker::new("late");
        drop(early);
    }
    take_log()
}

/// Where a value is cleaned up once it has been handed to a function.
///
/// # Examples
///
/// `handed_over()` returns `["given", "kept"]`.
pub fn handed_over() -> Vec<String> {
    {
        let given = Tracker::new("given");
        consume(given);
        let _kept = Tracker::new("kept");
    }
    take_log()
}

/// The order a collection cleans its elements up in.
///
/// # Examples
///
/// `from_a_vec(vec!["a".to_string(), "b".to_string()])` returns `["a", "b"]`.
/// `from_a_vec(vec![])` returns `[]`.
pub fn from_a_vec(names: Vec<String>) -> Vec<String> {
    {
        let mut group = Vec::with_capacity(names.len());
        for name in names {
            group.push(Tracker::new(&name));
        }
    }
    take_log()
}

/// Three values cleaned up in an order you choose.
///
/// # Examples
///
/// `custom_order()` returns `["b", "a", "c"]`.
pub fn custom_order() -> Vec<String> {
    {
        let a = Tracker::new("a");
        let b = Tracker::new("b");
        let _c = Tracker::new("c");
        drop(b);
        drop(a);
    }
    take_log()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_cleans_up_in_reverse_declaration_order() {
        assert_eq!(scope_order(), ["c", "b", "a"]);
    }

    #[test]
    fn an_early_release_comes_before_the_end_of_the_block() {
        assert_eq!(release_early(), ["early", "late"]);
    }

    #[test]
    fn a_value_given_away_is_cleaned_up_by_whoever_took_it() {
        assert_eq!(handed_over(), ["given", "kept"]);
    }

    #[test]
    fn a_vec_cleans_its_elements_up_front_to_back() {
        assert_eq!(
            from_a_vec(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ["a", "b", "c"]
        );
        assert_eq!(from_a_vec(vec!["only".to_string()]), ["only"]);
        assert_eq!(from_a_vec(Vec::new()), Vec::<String>::new());
    }

    #[test]
    fn the_order_can_be_chosen() {
        assert_eq!(custom_order(), ["b", "a", "c"]);
    }
}
