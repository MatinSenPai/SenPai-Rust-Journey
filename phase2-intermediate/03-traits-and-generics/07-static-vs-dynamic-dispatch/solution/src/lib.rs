//! Exercises for 2.3.7 — static vs. dynamic dispatch, and object safety.
//!
//! `Summarize`, `AnimeSeries`, `MangaVolume`, and the `Fibonacci` iterator
//! (the exact type 2.2.4 built) are provided below, fully written — none of
//! that is this lesson's subject. What you write is the dispatch mechanics
//! around them: which functions get one compiled copy, which get many, and
//! which type can finally stop naming itself.

pub trait Summarize {
    fn summary(&self) -> String;
}

pub struct AnimeSeries {
    pub title: String,
    pub episodes: u32,
}

impl Summarize for AnimeSeries {
    fn summary(&self) -> String {
        format!("{} - {} episodes", self.title, self.episodes)
    }
}

pub struct MangaVolume {
    pub title: String,
    pub chapters: u32,
}

impl Summarize for MangaVolume {
    fn summary(&self) -> String {
        format!("{} - {} chapters", self.title, self.chapters)
    }
}

/// The combined length, in bytes, of every item's `summary()` string,
/// added together. Static dispatch: `T` is fixed to one concrete type for
/// the whole slice.
///
/// # Examples
///
/// For `items` holding one `AnimeSeries` with `summary()` equal to
/// `"Trigun - 26 episodes"` (20 bytes) and one more with `summary()` equal
/// to `"Baccano! - 13 episodes"` (22 bytes), this returns `42`.
pub fn total_summary_length_generic<T: Summarize>(items: &[T]) -> usize {
    items.iter().map(|item| item.summary().len()).sum()
}

/// The same total as `total_summary_length_generic`, computed the same
/// way, but over trait objects instead of one fixed concrete type —
/// dynamic dispatch.
pub fn total_summary_length_dyn(items: &[Box<dyn Summarize>]) -> usize {
    items.iter().map(|item| item.summary().len()).sum()
}

/// A two-element lineup holding `series` first and `volume` second, each
/// turned into the `Vec`'s element type so both fit in one collection
/// despite being different concrete structs.
///
/// # Examples
///
/// `lineup(series, volume)[0].summary()` equals `series.summary()`, and
/// `lineup(series, volume)[1].summary()` equals `volume.summary()`.
pub fn lineup(series: AnimeSeries, volume: MangaVolume) -> Vec<Box<dyn Summarize>> {
    vec![Box::new(series), Box::new(volume)]
}

/// The exact `Fibonacci` type 2.2.4 built: `next()` reads `current`,
/// computes what replaces it, and returns the value it read.
pub struct Fibonacci {
    current: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let value = self.current;
        let new_next = self.current + self.next;
        self.current = self.next;
        self.next = new_next;
        Some(value)
    }
}

/// A fresh Fibonacci sequence, starting `0, 1, 1, 2, 3, ...` — the same
/// starting values 2.2.4 used. The return type names only the promise
/// ("some `Iterator` of `u64`"), never the real `Fibonacci` type.
pub fn fibonacci() -> impl Iterator<Item = u64> {
    Fibonacci {
        current: 0,
        next: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_total_sums_summary_lengths() {
        let series = vec![
            AnimeSeries {
                title: "Trigun".to_string(),
                episodes: 26,
            },
            AnimeSeries {
                title: "Baccano!".to_string(),
                episodes: 13,
            },
        ];
        let expected: usize = series.iter().map(|s| s.summary().len()).sum();
        assert_eq!(total_summary_length_generic(&series), expected);
    }

    #[test]
    fn generic_total_of_empty_slice_is_zero() {
        let empty: Vec<AnimeSeries> = Vec::new();
        assert_eq!(total_summary_length_generic(&empty), 0);
    }

    #[test]
    fn dyn_total_matches_generic_total_for_the_same_items() {
        let boxed: Vec<Box<dyn Summarize>> = vec![
            Box::new(AnimeSeries {
                title: "Trigun".to_string(),
                episodes: 26,
            }),
            Box::new(MangaVolume {
                title: "Blame!".to_string(),
                chapters: 10,
            }),
        ];
        let expected: usize = boxed.iter().map(|i| i.summary().len()).sum();
        assert_eq!(total_summary_length_dyn(&boxed), expected);
    }

    #[test]
    fn lineup_holds_series_first_then_volume() {
        let series = AnimeSeries {
            title: "Trigun".to_string(),
            episodes: 26,
        };
        let volume = MangaVolume {
            title: "Blame!".to_string(),
            chapters: 10,
        };
        let series_summary = series.summary();
        let volume_summary = volume.summary();

        let result = lineup(series, volume);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].summary(), series_summary);
        assert_eq!(result[1].summary(), volume_summary);
    }

    #[test]
    fn fibonacci_matches_the_known_sequence() {
        let first_eight: Vec<u64> = fibonacci().take(8).collect();
        assert_eq!(first_eight, vec![0, 1, 1, 2, 3, 5, 8, 13]);
    }

    #[test]
    fn fibonacci_adapters_work_for_free() {
        let evens: Vec<u64> = fibonacci().take(10).filter(|n| n % 2 == 0).collect();
        assert_eq!(evens, vec![0, 2, 8, 34]);
    }
}
