//! Exercises for 1.5.3 — Enums as data.
//!
//! Every one of these either *builds* an `Entry` or asks a one-line question
//! about one. Taking an `Entry` apart properly — reaching inside a variant
//! and using what it carries — is 1.5.4.

/// One title in the library, in exactly one of four states.
///
/// Each variant carries what that state needs, and nothing else. There is no
/// `Planned` that secretly holds a score.
#[derive(Debug, PartialEq)]
pub enum Entry {
    /// Queued. Nothing watched yet, so there is nothing to carry.
    Planned,
    /// In progress, carrying the last episode watched.
    Watching(u32),
    /// Finished and scored, carrying the score out of ten.
    Rated { score: u8 },
    /// Given up on, carrying where it stopped and why.
    Dropped { episode: u32, reason: String },
}

/// The entry for a title with `watched` episodes behind it.
///
/// `watched == 0` means nothing has been started, so the answer is
/// `Entry::Planned`. Any other number means the title is in progress, and the
/// answer is `Entry::Watching` carrying that same number.
///
/// # Examples
///
/// `from_episode(0)` returns `Entry::Planned`.
/// `from_episode(7)` returns `Entry::Watching(7)`.
pub fn from_episode(watched: u32) -> Entry {
    todo!("decide which of the two states `watched` describes, and build it")
}

/// A finished entry carrying `score` out of ten.
///
/// A score from 0 to 10 is stored unchanged. Anything above 10 is stored as
/// 10, because the scale stops there.
///
/// # Examples
///
/// `rate(9)` returns `Entry::Rated { score: 9 }`.
/// `rate(0)` returns `Entry::Rated { score: 0 }`.
/// `rate(200)` returns `Entry::Rated { score: 10 }`.
pub fn rate(score: u8) -> Entry {
    todo!("build the finished state, keeping the score inside the scale")
}

/// An abandoned entry: where it stopped, and why.
///
/// `episode` is stored unchanged. An empty `reason` is stored as the exact
/// text `no reason given`; any other reason is stored unchanged.
///
/// # Examples
///
/// `drop_at(3, "too slow".to_string())` returns
/// `Entry::Dropped { episode: 3, reason: "too slow" }`.
/// `drop_at(3, String::new())` returns
/// `Entry::Dropped { episode: 3, reason: "no reason given" }`.
pub fn drop_at(episode: u32, reason: String) -> Entry {
    todo!("build the abandoned state, standing in for a reason nobody gave")
}

impl Entry {
    /// Whether this entry is in progress.
    ///
    /// True for `Entry::Watching`, whatever episode it carries. False for
    /// every other variant.
    ///
    /// # Examples
    ///
    /// `Entry::Watching(1).is_watching()` is `true`.
    /// `Entry::Watching(0).is_watching()` is `true`.
    /// `Entry::Planned.is_watching()` is `false`.
    pub fn is_watching(&self) -> bool {
        todo!("report whether this is the in-progress state")
    }

    /// Whether this entry is one of the good ones.
    ///
    /// True only for `Entry::Rated` carrying a score of 8 or more. A `Rated`
    /// carrying less than 8 is false, and so is every other variant.
    ///
    /// # Examples
    ///
    /// `Entry::Rated { score: 8 }.is_favourite()` is `true`.
    /// `Entry::Rated { score: 7 }.is_favourite()` is `false`.
    /// `Entry::Planned.is_favourite()` is `false`.
    pub fn is_favourite(&self) -> bool {
        todo!("report whether this is a rated entry scoring at least eight")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_watched_is_planned() {
        assert_eq!(from_episode(0), Entry::Planned);
    }

    #[test]
    fn any_episode_watched_is_in_progress() {
        assert_eq!(from_episode(1), Entry::Watching(1));
        assert_eq!(from_episode(7), Entry::Watching(7));
        assert_eq!(from_episode(1_000), Entry::Watching(1_000));
    }

    #[test]
    fn a_score_stays_inside_the_scale() {
        assert_eq!(rate(9), Entry::Rated { score: 9 });
        assert_eq!(rate(0), Entry::Rated { score: 0 });
        assert_eq!(rate(10), Entry::Rated { score: 10 });
        assert_eq!(rate(11), Entry::Rated { score: 10 });
        assert_eq!(rate(200), Entry::Rated { score: 10 });
    }

    #[test]
    fn an_empty_reason_gets_a_stand_in() {
        assert_eq!(
            drop_at(3, "too slow".to_string()),
            Entry::Dropped {
                episode: 3,
                reason: "too slow".to_string(),
            }
        );
        assert_eq!(
            drop_at(0, String::new()),
            Entry::Dropped {
                episode: 0,
                reason: "no reason given".to_string(),
            }
        );
    }

    #[test]
    fn only_watching_is_watching() {
        assert!(Entry::Watching(1).is_watching());
        assert!(Entry::Watching(0).is_watching());
        assert!(!Entry::Planned.is_watching());
        assert!(!Entry::Rated { score: 9 }.is_watching());
        assert!(!Entry::Dropped {
            episode: 3,
            reason: "too slow".to_string(),
        }
        .is_watching());
    }

    #[test]
    fn a_favourite_is_rated_eight_or_better() {
        assert!(Entry::Rated { score: 8 }.is_favourite());
        assert!(Entry::Rated { score: 10 }.is_favourite());
        assert!(!Entry::Rated { score: 7 }.is_favourite());
        assert!(!Entry::Rated { score: 0 }.is_favourite());
        assert!(!Entry::Planned.is_favourite());
        assert!(!Entry::Watching(9).is_favourite());
    }
}
