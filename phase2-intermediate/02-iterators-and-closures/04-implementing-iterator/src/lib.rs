//! Exercises for 2.2.4 — implementing `Iterator` and `IntoIterator`.
//!
//! Two small types, two traits. Once each `impl` block below is complete,
//! every adapter and consumer the standard library ships — `.map()`,
//! `.filter()`, `.collect()`, `.sum()`, all of it — already works on it.
//! Nothing else in this file needs writing for that to be true.

/// A Collatz-sequence generator.
///
/// Starting from a value, each step follows one rule: half of the current
/// value if it is even, or three times the current value plus one if it is
/// odd. Reaching `1` ends the sequence.
pub struct Collatz {
    current: Option<u64>,
}

impl Collatz {
    /// Starts a new sequence at `start`. `start` must be at least `1`; this
    /// type does not define what happens for `0`.
    pub fn new(start: u64) -> Self {
        Collatz {
            current: Some(start),
        }
    }
}

impl Iterator for Collatz {
    type Item = u64;

    /// Produces the Collatz sequence starting at the value passed to
    /// `Collatz::new`, one step per call:
    ///
    /// - Call the value this method is about to produce `v`. The value it
    ///   produces on the *next* call is `v / 2` if `v` is even, or
    ///   `3 * v + 1` if `v` is odd — except:
    /// - the value `1` is produced exactly once, and every call after that
    ///   returns `None` forever. The sequence does not repeat past `1`.
    ///
    /// # Examples
    ///
    /// Starting at `6`, successive calls produce, in order: `6`, `3`, `10`,
    /// `5`, `16`, `8`, `4`, `2`, `1`, and then `None` on every call after
    /// that. Starting at `1` produces just `1`, then `None`.
    fn next(&mut self) -> Option<u64> {
        todo!(
            "read the value stored as `current`; if there is none, the sequence already \
             finished, return None. Otherwise work out the value that replaces it, per the rule \
             in this doc comment (storing None once that value would be 1), and return the value \
             you originally read, wrapped in Some"
        )
    }
}

/// A logged sequence of episode titles, in the order they were watched.
pub struct EpisodeLog(pub Vec<String>);

impl IntoIterator for EpisodeLog {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    /// Consumes `self` and hands back an iterator over its titles, by
    /// value, in their original order — the same order they were stored
    /// in `self.0`.
    fn into_iter(self) -> Self::IntoIter {
        todo!("turn the Vec<String> stored inside `self` into the iterator this method returns")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collatz_from_six_matches_the_known_sequence() {
        let sequence: Vec<u64> = Collatz::new(6).collect();
        assert_eq!(sequence, vec![6, 3, 10, 5, 16, 8, 4, 2, 1]);
    }

    #[test]
    fn collatz_from_one_is_just_one() {
        let sequence: Vec<u64> = Collatz::new(1).collect();
        assert_eq!(sequence, vec![1]);
    }

    #[test]
    fn collatz_returns_none_forever_once_finished() {
        let mut sequence = Collatz::new(2);
        assert_eq!(sequence.next(), Some(2));
        assert_eq!(sequence.next(), Some(1));
        assert_eq!(sequence.next(), None);
        assert_eq!(sequence.next(), None);
    }

    #[test]
    fn collatz_works_with_adapters_nobody_wrote_here() {
        let even_steps = Collatz::new(6).filter(|value| value % 2 == 0).count();
        assert_eq!(even_steps, 6);
    }

    #[test]
    fn episode_log_into_iter_yields_titles_in_order() {
        let log = EpisodeLog(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);
        let titles: Vec<String> = log.into_iter().collect();
        assert_eq!(
            titles,
            vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]
        );
    }

    #[test]
    fn episode_log_for_loop_compiles_and_moves_it() {
        let log = EpisodeLog(vec!["Mushoku Tensei".to_string()]);
        let mut seen = Vec::new();
        for title in log {
            seen.push(title);
        }
        assert_eq!(seen, vec!["Mushoku Tensei".to_string()]);
    }

    #[test]
    fn episode_log_adapters_work_for_free() {
        let log = EpisodeLog(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        let kept: Vec<String> = log.into_iter().filter(|title| title != "B").collect();
        assert_eq!(kept, vec!["A".to_string(), "C".to_string()]);
    }
}
