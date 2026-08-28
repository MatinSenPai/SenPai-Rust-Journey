//! Exercises for 2.6.3 — `Rc` and `Arc`.
//!
//! `AppConfig` is provided below, fully written — designing it is not this
//! lesson's subject. What you write is the shared-ownership mechanics
//! around it: wrapping it so it can have more than one owner, adding an
//! owner without copying its fields, reading how many owners currently
//! exist, and telling "the same allocation" apart from "an equal value."

use std::rc::Rc;
use std::sync::Arc;

/// A small piece of data multiple independent parts of a program might all
/// need to read at once — a request handler, a background job, and a
/// logger, none of which is obviously "the" single owner.
#[derive(Debug, PartialEq)]
pub struct AppConfig {
    pub app_name: String,
    pub max_connections: u32,
}

/// Wraps `config` in an `Rc`, so it can have more than one owner.
///
/// # Examples
///
/// For `config.app_name` equal to `"senpai-api"`, the returned `Rc`'s
/// `app_name` field reads back the same `"senpai-api"`, and
/// `owner_count` on it is `1`.
pub fn share_config(config: AppConfig) -> Rc<AppConfig> {
    Rc::new(config)
}

/// Hands back a second `Rc` pointing at the exact same allocation as
/// `shared` — this must not copy `AppConfig`'s fields, only add an owner.
///
/// # Examples
///
/// After `let second = add_owner(&shared);`, `owner_count(&shared)` is one
/// higher than it was before, and `same_allocation(&shared, &second)` is
/// `true`.
pub fn add_owner(shared: &Rc<AppConfig>) -> Rc<AppConfig> {
    Rc::clone(shared)
}

/// How many `Rc` handles currently point at the same allocation as
/// `shared`.
pub fn owner_count(shared: &Rc<AppConfig>) -> usize {
    Rc::strong_count(shared)
}

/// `true` exactly when `a` and `b` point at the same heap allocation — not
/// merely two `AppConfig`s with equal fields, but the very same one.
///
/// # Examples
///
/// For `let b = add_owner(&a);`, `same_allocation(&a, &b)` is `true`. For
/// two separately built `Rc`s that happen to hold equal `AppConfig`
/// values, it is `false`.
pub fn same_allocation(a: &Rc<AppConfig>, b: &Rc<AppConfig>) -> bool {
    Rc::ptr_eq(a, b)
}

/// Same idea as `share_config`, but with the thread-safe `Arc`.
pub fn share_config_across_threads(config: AppConfig) -> Arc<AppConfig> {
    Arc::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> AppConfig {
        AppConfig {
            app_name: "senpai-api".to_string(),
            max_connections: 100,
        }
    }

    #[test]
    fn shares_config_with_owner_count_one() {
        let shared = share_config(sample_config());
        assert_eq!(owner_count(&shared), 1);
    }

    #[test]
    fn adding_an_owner_increments_the_count_and_reads_the_same_data() {
        let shared = share_config(sample_config());
        let second = add_owner(&shared);

        assert_eq!(owner_count(&shared), 2);
        assert_eq!(second.app_name, "senpai-api");
        assert_eq!(second.max_connections, 100);
    }

    #[test]
    fn dropping_an_owner_decrements_the_count() {
        let shared = share_config(sample_config());
        assert_eq!(owner_count(&shared), 1);

        {
            let _second = add_owner(&shared);
            assert_eq!(owner_count(&shared), 2);
        } // _second dropped here, at the end of this inner scope

        assert_eq!(owner_count(&shared), 1);
    }

    #[test]
    fn same_allocation_is_true_for_an_added_owner() {
        let shared = share_config(sample_config());
        let second = add_owner(&shared);

        assert!(same_allocation(&shared, &second));
    }

    #[test]
    fn same_allocation_is_false_for_two_equal_but_separate_rcs() {
        let a = share_config(sample_config());
        let b = share_config(sample_config());

        assert_eq!(*a, *b, "the two configs should hold equal data");
        assert!(!same_allocation(&a, &b));
    }

    #[test]
    fn share_config_across_threads_wraps_the_same_data() {
        let shared = share_config_across_threads(sample_config());
        assert_eq!(shared.app_name, "senpai-api");
        assert_eq!(shared.max_connections, 100);
        assert_eq!(Arc::strong_count(&shared), 1);
    }
}
