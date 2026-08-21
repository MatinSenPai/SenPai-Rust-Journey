//! Exercises for 1.5.5 — `if let`, `while let`, `let ... else`.
//!
//! Each one is here because a particular form is the clearest way to write
//! it. Every one of them *can* be written as a `match`; write the `match`
//! afterwards and compare the two.

/// One line of a watchlist.
#[derive(Debug, PartialEq)]
pub enum Entry {
    /// Being watched, and how far in.
    Watching { episode: u32 },
    /// Finished, with a score out of ten.
    Completed { rating: u8 },
    /// On the list, not started.
    PlanToWatch,
    /// Abandoned, and at which episode.
    Dropped { at_episode: u32 },
}

/// Where `entry` has got to, as a line of text.
///
/// For a `Watching` entry the answer is `"on episode "` followed by its
/// episode number, so episode 12 gives exactly `"on episode 12"`.
///
/// For every other variant the answer is exactly `"not watching"`.
///
/// # Examples
///
/// `watching_line(&Entry::Watching { episode: 12 })` is `"on episode 12"`.
/// `watching_line(&Entry::PlanToWatch)` is `"not watching"`.
pub fn watching_line(entry: &Entry) -> String {
    todo!("say which episode this entry is on, or that it is not being watched")
}

/// The rating of every `Completed` entry in `entries`, in the order they
/// appear.
///
/// Entries of any other shape contribute nothing at all — they are not
/// skipped over with a placeholder, they are simply absent from the answer.
///
/// # Examples
///
/// Given `[Completed { rating: 9 }, PlanToWatch, Completed { rating: 4 }]`
/// the answer is `[9, 4]`.
/// Given `[]` the answer is `[]`.
pub fn ratings_only(entries: Vec<Entry>) -> Vec<u8> {
    todo!("keep the score of every finished entry, in order")
}

/// Everything in `stack`, in the order `Vec::pop` hands it back — the last
/// one in comes out first.
///
/// # Examples
///
/// `pop_all(vec![1, 2, 3])` is `[3, 2, 1]`.
/// `pop_all(vec![7])` is `[7]`.
/// `pop_all(vec![])` is `[]`.
pub fn pop_all(stack: Vec<i32>) -> Vec<i32> {
    todo!("empty the stack, keeping what comes out in the order it came out")
}

/// How many episodes behind `latest_episode` this entry is.
///
/// For a `Watching` entry the answer is `latest_episode` minus its episode.
/// When the entry is somehow *ahead* of `latest_episode` the answer is `0`
/// — it never wraps around.
///
/// For every other variant the answer is `0`.
///
/// # Examples
///
/// `episode_gap(&Entry::Watching { episode: 7 }, 12)` is `5`.
/// `episode_gap(&Entry::Watching { episode: 14 }, 12)` is `0`.
/// `episode_gap(&Entry::PlanToWatch, 12)` is `0`.
pub fn episode_gap(entry: &Entry, latest_episode: u32) -> u32 {
    todo!("work out how many episodes behind this entry is")
}

/// What comes out of `stack` before the first negative number does.
///
/// Values are popped one at a time and kept, in the order they came out,
/// until a negative one appears. That negative number is not kept, and
/// nothing underneath it is looked at. Zero is not negative.
///
/// # Examples
///
/// `pop_until_negative(vec![1, -2, 3, 4])` is `[4, 3]`.
/// `pop_until_negative(vec![1, 2])` is `[2, 1]`.
/// `pop_until_negative(vec![-1])` is `[]`.
/// `pop_until_negative(vec![])` is `[]`.
pub fn pop_until_negative(stack: Vec<i32>) -> Vec<i32> {
    todo!("take from the stack until a negative number comes out, keeping the rest")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_episode_or_says_it_is_not_watching() {
        assert_eq!(
            watching_line(&Entry::Watching { episode: 12 }),
            "on episode 12"
        );
        assert_eq!(
            watching_line(&Entry::Watching { episode: 1 }),
            "on episode 1"
        );
        assert_eq!(watching_line(&Entry::PlanToWatch), "not watching");
        assert_eq!(
            watching_line(&Entry::Completed { rating: 9 }),
            "not watching"
        );
        assert_eq!(
            watching_line(&Entry::Dropped { at_episode: 3 }),
            "not watching"
        );
    }

    #[test]
    fn keeps_only_the_finished_ratings() {
        let shelf = vec![
            Entry::Completed { rating: 9 },
            Entry::PlanToWatch,
            Entry::Watching { episode: 4 },
            Entry::Completed { rating: 4 },
        ];
        assert_eq!(ratings_only(shelf), vec![9, 4]);

        assert_eq!(ratings_only(vec![]), Vec::<u8>::new());
        assert_eq!(ratings_only(vec![Entry::PlanToWatch]), Vec::<u8>::new());
    }

    #[test]
    fn empties_the_stack_last_in_first_out() {
        assert_eq!(pop_all(vec![1, 2, 3]), vec![3, 2, 1]);
        assert_eq!(pop_all(vec![7]), vec![7]);
        assert_eq!(pop_all(vec![]), Vec::<i32>::new());
    }

    #[test]
    fn measures_the_gap_and_never_wraps() {
        assert_eq!(episode_gap(&Entry::Watching { episode: 7 }, 12), 5);
        assert_eq!(episode_gap(&Entry::Watching { episode: 12 }, 12), 0);
        assert_eq!(episode_gap(&Entry::Watching { episode: 14 }, 12), 0);
        assert_eq!(episode_gap(&Entry::PlanToWatch, 12), 0);
        assert_eq!(episode_gap(&Entry::Completed { rating: 9 }, 12), 0);
        assert_eq!(episode_gap(&Entry::Dropped { at_episode: 3 }, 12), 0);
    }

    #[test]
    fn stops_at_the_first_negative() {
        assert_eq!(pop_until_negative(vec![1, -2, 3, 4]), vec![4, 3]);
        assert_eq!(pop_until_negative(vec![1, 2]), vec![2, 1]);
        assert_eq!(pop_until_negative(vec![-1]), Vec::<i32>::new());
        assert_eq!(pop_until_negative(vec![]), Vec::<i32>::new());
        assert_eq!(pop_until_negative(vec![5, 0, 6]), vec![6, 0, 5]);
    }
}
