//! `Tracker` and the drop-log below are provided for you — they use a
//! couple of tools (`thread_local!`, `RefCell`) from later lessons just to
//! make drop order observable in a test. You don't need to understand their
//! internals; treat them like test infrastructure. Your job is the three
//! `todo!()` functions below, which only ever call `Tracker::new(name)`,
//! ordinary scoping (`{ ... }`), and `drop(value)`.

use std::cell::RefCell;

thread_local! {
    static DROP_LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn log_drop(name: &str) {
    DROP_LOG.with(|log| log.borrow_mut().push(name.to_string()));
}

/// Drains and returns everything logged so far on this thread.
fn take_log() -> Vec<String> {
    DROP_LOG.with(|log| log.borrow_mut().drain(..).collect())
}

pub struct Tracker {
    name: String,
}

impl Tracker {
    pub fn new(name: &str) -> Self {
        Tracker {
            name: name.to_string(),
        }
    }
}

impl Drop for Tracker {
    fn drop(&mut self) {
        log_drop(&self.name);
    }
}

/// Create three `Tracker`s, named "a", "b", "c" in that order, all inside
/// one inner scope (`{ ... }`), and nothing else. Return the log
/// afterward — it should show them dropped in **reverse** declaration
/// order.
pub fn drop_order_in_one_scope() -> Vec<String> {
    take_log(); // clears any leftovers from a previous test on this thread
    todo!("create Tracker::new(\"a\"), \"b\", \"c\" inside a `{{ }}` block, then return take_log()")
}

/// Create `Tracker::new("first")` and `Tracker::new("second")` inside one
/// inner scope. Explicitly `drop(first)` *before* the scope ends, so
/// "first" is logged before "second" (which drops naturally at the end of
/// the scope) — the opposite order you'd get without the explicit early
/// drop.
pub fn early_drop_demo() -> Vec<String> {
    take_log();
    todo!("create both trackers, `drop(first)` explicitly, let second drop naturally, return take_log()")
}

fn create_tracker(name: &str) -> Tracker {
    // Note: this Tracker is NOT dropped here, even though its creation
    // happens entirely inside this function's body — it's returned, so
    // ownership (and therefore *when* it drops) moves to the caller.
    Tracker::new(name)
}

/// Call `create_tracker("moved")` inside an inner scope, and confirm (by
/// checking the log) that it only gets dropped once the *outer* scope
/// ends — not inside `create_tracker` itself.
pub fn move_extends_lifetime() -> Vec<String> {
    take_log();
    todo!("let _t = create_tracker(\"moved\"); inside a `{{ }}` block, then return take_log()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_in_reverse_declaration_order() {
        assert_eq!(
            drop_order_in_one_scope(),
            vec!["c".to_string(), "b".to_string(), "a".to_string()]
        );
    }

    #[test]
    fn early_drop_reorders() {
        assert_eq!(
            early_drop_demo(),
            vec!["first".to_string(), "second".to_string()]
        );
    }

    #[test]
    fn move_delays_drop_until_outer_scope_ends() {
        assert_eq!(move_extends_lifetime(), vec!["moved".to_string()]);
    }
}
