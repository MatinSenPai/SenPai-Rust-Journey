//! Exercises for 1.2.5 — `Drop` and RAII.
//!
//! Everything down to the first `todo!()` is scaffolding, and you never have
//! to change it: `Tracker` is a value whose cleanup is *visible*, because
//! dropping one writes its name into a log the tests can read back. Making a
//! global log writable needs one tool from Phase 2 (`RefCell`); treat that
//! block as test equipment rather than as something to understand today.
//!
//! Your five functions need nothing but `Tracker::new`, ordinary blocks,
//! `drop`, and `take_log`.

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
/// Create `Tracker`s named `"a"`, `"b"` and `"c"`, in that order, inside one
/// inner block, all three alive until that block ends. Then return the log.
///
/// # Examples
///
/// `scope_order()` returns `["c", "b", "a"]`.
pub fn scope_order() -> Vec<String> {
    todo!("hold three trackers named a, b and c in one block, then hand back the log")
}

/// Two values in one block, the first of them let go before the second.
///
/// Create a `Tracker` named `"early"` and then one named `"late"`, in that
/// order, inside one inner block. See to it that `"early"` is cleaned up
/// before `"late"` — without swapping the two lines and without giving either
/// a block of its own. Then return the log.
///
/// # Examples
///
/// `release_early()` returns `["early", "late"]`.
pub fn release_early() -> Vec<String> {
    todo!("hold two trackers, early and late, and let go of the first before the block ends")
}

/// Where a value is cleaned up once it has been handed to a function.
///
/// Inside one inner block: create a `Tracker` named `"given"` and hand it to
/// `consume`, then create one named `"kept"` and leave it alone until the
/// block ends. Then return the log.
///
/// # Examples
///
/// `handed_over()` returns `["given", "kept"]` — the first was cleaned up
/// inside `consume`, before the second one existed.
pub fn handed_over() -> Vec<String> {
    todo!("give one tracker away to a function and keep a second one to the end of the block")
}

/// The order a collection cleans its elements up in.
///
/// Build one `Tracker` per name in `names`, in order, all of them alive at the
/// same time in one `Vec`, inside one inner block. Then return the log.
///
/// A `Vec` is not a pile of `let` bindings: it cleans its elements up front to
/// back, so the answer comes back in the same order as `names` rather than
/// reversed.
///
/// # Examples
///
/// `from_a_vec(vec!["a".to_string(), "b".to_string()])` returns `["a", "b"]`.
/// `from_a_vec(vec![])` returns `[]`.
pub fn from_a_vec(names: Vec<String>) -> Vec<String> {
    todo!("keep one tracker per name alive together in a Vec, then hand back the log")
}

/// Three values cleaned up in an order you choose.
///
/// Create `Tracker`s named `"a"`, `"b"` and `"c"`, in that order, inside one
/// inner block — and see to it that they are cleaned up in the order `"b"`,
/// then `"a"`, then `"c"`. Do not swap the three lines and do not give any of
/// them a block of its own. Then return the log.
///
/// # Examples
///
/// `custom_order()` returns `["b", "a", "c"]`.
pub fn custom_order() -> Vec<String> {
    todo!("hold three trackers named a, b and c, and end them in the order b, a, c")
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
