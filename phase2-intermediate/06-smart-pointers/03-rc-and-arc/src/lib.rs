use std::rc::Rc;
use std::sync::Arc;

/// A small config value, the kind of thing multiple independent parts of a
/// program might need to read at once (a request handler, a background job,
/// a logger — none of which is obviously "the" single owner).
pub struct AppConfig {
    pub app_name: String,
    pub max_connections: u32,
}

/// Wraps `config` in an `Rc`, making it shareable (single-threaded) among
/// multiple owners.
pub fn share_config(config: AppConfig) -> Rc<AppConfig> {
    todo!("Rc::new(config)")
}

/// Produces a second handle to the *same* underlying `AppConfig` — this does
/// not copy the config's data, it just increments the internal count.
pub fn clone_handle(shared: &Rc<AppConfig>) -> Rc<AppConfig> {
    todo!("Rc::clone(shared)")
}

/// Returns how many `Rc` handles currently point at the same value as
/// `shared`.
pub fn reference_count(shared: &Rc<AppConfig>) -> usize {
    todo!("Rc::strong_count(shared)")
}

/// Same idea as `share_config`, but with the thread-safe `Arc` — needed the
/// moment a value has to cross a `std::thread::spawn` boundary, since `Rc`
/// is not `Send`.
pub fn share_config_across_threads(config: AppConfig) -> Arc<AppConfig> {
    todo!("Arc::new(config)")
}

#[cfg(test)]
mod tests {
    use super::*;
    // Only used inside this test module (only `arc_can_be_shared_across_spawned_threads`
    // needs it) — importing it here instead of at the top of the file avoids an
    // `unused_imports` warning on a plain (non-test) `cargo build`, since `#[cfg(test)]`
    // code isn't compiled at all for that build, so an import only consumed by tests
    // genuinely looks unused from a non-test build's point of view.
    use std::thread;

    fn sample_config() -> AppConfig {
        AppConfig {
            app_name: "senpai-api".to_string(),
            max_connections: 100,
        }
    }

    #[test]
    fn shares_config_with_count_one() {
        let shared = share_config(sample_config());
        assert_eq!(reference_count(&shared), 1);
    }

    #[test]
    fn cloning_increments_count_and_reads_same_data() {
        let shared = share_config(sample_config());
        let handle = clone_handle(&shared);

        assert_eq!(reference_count(&shared), 2);
        assert_eq!(handle.app_name, "senpai-api");
        assert_eq!(handle.max_connections, 100);
    }

    #[test]
    fn dropping_a_clone_decrements_the_count() {
        let shared = share_config(sample_config());
        assert_eq!(reference_count(&shared), 1);

        {
            let _handle = clone_handle(&shared);
            assert_eq!(reference_count(&shared), 2);
        } // _handle dropped here, at the end of this inner scope

        assert_eq!(reference_count(&shared), 1);
    }

    #[test]
    fn arc_can_be_shared_across_spawned_threads() {
        let shared = share_config_across_threads(sample_config());

        let mut handles = Vec::new();
        for _ in 0..3 {
            let shared = Arc::clone(&shared);
            handles.push(thread::spawn(move || shared.max_connections));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().expect("thread should not panic"));
        }

        assert_eq!(results, vec![100, 100, 100]);
    }
}
