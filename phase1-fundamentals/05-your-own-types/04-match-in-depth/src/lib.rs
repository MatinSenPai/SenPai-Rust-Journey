//! Exercises for 1.5.4 — `match` in depth.
//!
//! One enum, six functions. Every one of them is a `match`, and each asks for
//! a different piece of the pattern language: plain arms, ranges and
//! alternatives, guards, an `@` binding, a tuple of two values, and an
//! `Option` — which is an ordinary enum and needs nothing new.
//!
//! ```sh
//! cargo test -p p1-05-04-match-in-depth
//! ```

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
///
/// The numbers in the table are examples; the real values come from the
/// variant being described.
pub fn describe(progress: &Progress) -> String {
    todo!("one arm per variant, each building the line the table asks for")
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
    todo!("group the scores into the six bands above")
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
///
/// The `Reading` rows are the interesting ones: no pattern on its own can say
/// "these two fields are equal".
pub fn shelf(progress: &Progress) -> String {
    todo!("sort a progress onto one of the six shelves above")
}

/// A label for a chapter number, with the number itself in it.
///
/// | `chapter` | answer |
/// |---|---|
/// | `0` | `"no chapters yet"` |
/// | `1`–`9` | `"early (chapter 4)"` |
/// | `10`–`99` | `"mid (chapter 40)"` |
/// | `100` and up | `"long runner (chapter 327)"` |
///
/// Again the numbers shown are examples: the answer carries whatever
/// `chapter` was passed in.
pub fn chapter_label(chapter: u32) -> String {
    todo!("name the stage this chapter number is in, and put the number in the answer")
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
///
/// The rows are tried in the order they are written here.
pub fn pair_verdict(mine: &Progress, theirs: &Progress) -> String {
    todo!("compare the two progresses in a single match and answer as the table says")
}

/// How far behind the release the reader is.
///
/// `latest` is the newest published chapter, or `None` when the series has
/// published nothing yet. `read` is how far this reader has got.
///
/// | case | answer |
/// |---|---|
/// | `None` | `"nothing published yet"` |
/// | `Some(n)` where `n` is `read` | `"caught up"` |
/// | `Some(n)` where `n` is above `read` | the gap, then `" behind"`: `"3 behind"` |
/// | `Some(n)` where `n` is below `read` | `"ahead of the release"` |
pub fn release_note(latest: Option<u32>, read: u32) -> String {
    todo!("answer as the table says; the third row needs the size of the gap")
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
