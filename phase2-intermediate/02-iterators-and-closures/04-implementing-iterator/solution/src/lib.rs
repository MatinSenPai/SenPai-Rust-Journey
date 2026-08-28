//! Solution for 2.2.4 — implementing `Iterator` and `IntoIterator`.

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

    fn next(&mut self) -> Option<u64> {
        let value = self.current?;
        self.current = if value == 1 {
            None
        } else if value % 2 == 0 {
            Some(value / 2)
        } else {
            Some(3 * value + 1)
        };
        Some(value)
    }
}

/// A logged sequence of episode titles, in the order they were watched.
pub struct EpisodeLog(pub Vec<String>);

impl IntoIterator for EpisodeLog {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
