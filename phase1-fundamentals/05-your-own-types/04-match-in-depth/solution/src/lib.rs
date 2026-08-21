//! Reference solution for 1.5.4 — `match` in depth.
//!
//! One enum, six functions, six different pieces of the pattern language.
//! Every `match` here is exhaustive, and three of them are exhaustive without
//! a `_` arm — which is the point: adding a variant to `Progress` should stop
//! the build until each of those three has been thought about again.

/// Where one series has got to.
#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    /// On the shelf, never opened.
    NotStarted,
    /// Part way through: `chapter` read, `of` published.
    Reading { chapter: u32, of: u32 },
    /// Done, and scored out of ten.
    Finished { rating: u8 },
    /// Given up at chapter `at`.
    Dropped { at: u32 },
}

/// One line describing `progress`.
///
/// | variant | answer |
/// |---|---|
/// | `NotStarted` | `"not started"` |
/// | `Reading { chapter, of }` | `"chapter 12 of 40"` |
/// | `Finished { rating }` | `"finished, 9/10"` |
/// | `Dropped { at }` | `"dropped at chapter 3"` |
pub fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at } => format!("dropped at chapter {at}"),
    }
}

/// The band a score falls into.
///
/// | `stars` | answer |
/// |---|---|
/// | `0` | `"unrated"` |
/// | `1`, `2`, `3` | `"weak"` |
/// | `4`, `5`, `6` | `"watchable"` |
/// | `7`, `8` | `"good"` |
/// | `9`, `10` | `"top shelf"` |
/// | anything above `10` | `"not a score"` |
pub fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}

/// Which shelf `progress` belongs on.
///
/// | case | answer |
/// |---|---|
/// | `Finished` with a rating of 9 or 10 | `"hall of fame"` |
/// | `Finished` with any other rating | `"read"` |
/// | `Reading` where `chapter` equals `of` | `"waiting for the next chapter"` |
/// | `Reading` otherwise | `"reading"` |
/// | `NotStarted` | `"the pile"` |
/// | `Dropped` | `"gone"` |
pub fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: 9 | 10 } => "hall of fame".to_string(),
        Progress::Finished { .. } => "read".to_string(),
        Progress::Reading { chapter, of } if chapter == of => {
            "waiting for the next chapter".to_string()
        }
        Progress::Reading { .. } => "reading".to_string(),
        Progress::NotStarted => "the pile".to_string(),
        Progress::Dropped { .. } => "gone".to_string(),
    }
}

/// A label for a chapter number, with the number itself in it.
///
/// | `chapter` | answer |
/// |---|---|
/// | `0` | `"no chapters yet"` |
/// | `1`–`9` | `"early (chapter 4)"` |
/// | `10`–`99` | `"mid (chapter 40)"` |
/// | `100` and up | `"long runner (chapter 327)"` |
pub fn chapter_label(chapter: u32) -> String {
    match chapter {
        0 => "no chapters yet".to_string(),
        n @ 1..=9 => format!("early (chapter {n})"),
        n @ 10..=99 => format!("mid (chapter {n})"),
        n => format!("long runner (chapter {n})"),
    }
}

/// What two readers' progress on the same series says about them.
///
/// | case | answer |
/// |---|---|
/// | both `Finished` with the same rating | `"we agree"` |
/// | both `Finished` with different ratings | `"we disagree"` |
/// | both `NotStarted` | `"neither of us has started"` |
/// | exactly one of the two is `Finished` | `"one of us finished"` |
/// | anything else | `"still reading"` |
pub fn pair_verdict(mine: &Progress, theirs: &Progress) -> String {
    use Progress::{Finished, NotStarted};
    match (mine, theirs) {
        (Finished { rating: a }, Finished { rating: b }) if a == b => "we agree".to_string(),
        (Finished { .. }, Finished { .. }) => "we disagree".to_string(),
        (NotStarted, NotStarted) => "neither of us has started".to_string(),
        (Finished { .. }, _) | (_, Finished { .. }) => "one of us finished".to_string(),
        _ => "still reading".to_string(),
    }
}

/// How far behind the release the reader is.
///
/// | case | answer |
/// |---|---|
/// | `None` | `"nothing published yet"` |
/// | `Some(n)` where `n` is `read` | `"caught up"` |
/// | `Some(n)` where `n` is above `read` | the gap, then `" behind"`: `"3 behind"` |
/// | `Some(n)` where `n` is below `read` | `"ahead of the release"` |
pub fn release_note(latest: Option<u32>, read: u32) -> String {
    match latest {
        None => "nothing published yet".to_string(),
        Some(n) if n == read => "caught up".to_string(),
        Some(n) if n > read => format!("{} behind", n - read),
        Some(_) => "ahead of the release".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_every_variant() {
        assert_eq!(describe(&Progress::NotStarted), "not started");
        assert_eq!(
            describe(&Progress::Reading {
                chapter: 12,
                of: 40
            }),
            "chapter 12 of 40"
        );
        assert_eq!(
            describe(&Progress::Finished { rating: 9 }),
            "finished, 9/10"
        );
        assert_eq!(
            describe(&Progress::Dropped { at: 3 }),
            "dropped at chapter 3"
        );
    }

    #[test]
    fn bands_every_score() {
        assert_eq!(band(0), "unrated");
        assert_eq!(band(1), "weak");
        assert_eq!(band(3), "weak");
        assert_eq!(band(4), "watchable");
        assert_eq!(band(6), "watchable");
        assert_eq!(band(7), "good");
        assert_eq!(band(8), "good");
        assert_eq!(band(9), "top shelf");
        assert_eq!(band(10), "top shelf");
        assert_eq!(band(11), "not a score");
        assert_eq!(band(255), "not a score");
    }

    #[test]
    fn sorts_onto_shelves() {
        assert_eq!(shelf(&Progress::Finished { rating: 10 }), "hall of fame");
        assert_eq!(shelf(&Progress::Finished { rating: 9 }), "hall of fame");
        assert_eq!(shelf(&Progress::Finished { rating: 8 }), "read");
        assert_eq!(shelf(&Progress::Finished { rating: 0 }), "read");
        assert_eq!(
            shelf(&Progress::Reading {
                chapter: 40,
                of: 40
            }),
            "waiting for the next chapter"
        );
        assert_eq!(
            shelf(&Progress::Reading {
                chapter: 12,
                of: 40
            }),
            "reading"
        );
        assert_eq!(shelf(&Progress::NotStarted), "the pile");
        assert_eq!(shelf(&Progress::Dropped { at: 3 }), "gone");
    }

    #[test]
    fn labels_chapter_numbers() {
        assert_eq!(chapter_label(0), "no chapters yet");
        assert_eq!(chapter_label(1), "early (chapter 1)");
        assert_eq!(chapter_label(9), "early (chapter 9)");
        assert_eq!(chapter_label(10), "mid (chapter 10)");
        assert_eq!(chapter_label(99), "mid (chapter 99)");
        assert_eq!(chapter_label(100), "long runner (chapter 100)");
        assert_eq!(chapter_label(327), "long runner (chapter 327)");
    }

    #[test]
    fn compares_two_readers() {
        let nine = Progress::Finished { rating: 9 };
        let four = Progress::Finished { rating: 4 };
        let reading = Progress::Reading {
            chapter: 12,
            of: 40,
        };

        assert_eq!(pair_verdict(&nine, &nine), "we agree");
        assert_eq!(pair_verdict(&nine, &four), "we disagree");
        assert_eq!(
            pair_verdict(&Progress::NotStarted, &Progress::NotStarted),
            "neither of us has started"
        );
        assert_eq!(pair_verdict(&nine, &reading), "one of us finished");
        assert_eq!(pair_verdict(&reading, &nine), "one of us finished");
        assert_eq!(pair_verdict(&reading, &reading), "still reading");
        assert_eq!(
            pair_verdict(&Progress::NotStarted, &reading),
            "still reading"
        );
        assert_eq!(
            pair_verdict(&Progress::Dropped { at: 3 }, &Progress::NotStarted),
            "still reading"
        );
    }

    #[test]
    fn reports_the_release_gap() {
        assert_eq!(release_note(None, 0), "nothing published yet");
        assert_eq!(release_note(None, 12), "nothing published yet");
        assert_eq!(release_note(Some(40), 40), "caught up");
        assert_eq!(release_note(Some(43), 40), "3 behind");
        assert_eq!(release_note(Some(1), 0), "1 behind");
        assert_eq!(release_note(Some(38), 40), "ahead of the release");
    }
}
