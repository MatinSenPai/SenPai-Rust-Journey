//! Matching a tuple compares two values in one place, matching through a `&`
//! binds references to the fields, and `Option` is an ordinary enum that needs
//! nothing new at all.
//!
//!     cargo run -p p1-05-04-match-in-depth --example 04-tuples-and-options

#[derive(Debug)]
enum Progress {
    NotStarted,
    Reading { chapter: u32 },
    Finished { rating: u8 },
}

/// Two values, one `match`. Written as a chain of `if`s this is nine
/// comparisons and a bug waiting in the last `else`.
fn compare(mine: &Progress, theirs: &Progress) -> String {
    use Progress::{Finished, NotStarted, Reading};
    match (mine, theirs) {
        (Finished { rating: a }, Finished { rating: b }) if a == b => {
            "we gave it the same score".to_string()
        }
        (Finished { rating: a }, Finished { rating: b }) => {
            format!("{a}/10 against {b}/10")
        }
        (NotStarted, NotStarted) => "neither of us has started".to_string(),
        (Finished { .. }, _) | (_, Finished { .. }) => "one of us has finished".to_string(),
        (Reading { chapter: a }, Reading { chapter: b }) => {
            format!("chapter {a} against chapter {b}")
        }
        _ => "still going".to_string(),
    }
}

/// `progress` is a `&Progress`, so a pattern that names a field binds a
/// reference to it: `chapter` here is a `&u32`. That is why the value comes
/// out with a `*`.
fn chapters_read(progress: &Progress) -> u32 {
    match progress {
        Progress::NotStarted => 0,
        Progress::Reading { chapter } => *chapter,
        Progress::Finished { .. } => 0,
    }
}

/// A wildcard on an enum. It compiles today, and it will still compile the day
/// a fourth variant is added — which is exactly the problem.
fn tag(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        _ => "not finished".to_string(),
    }
}

/// `Option<u32>` is `enum Option<T> { None, Some(T) }` and nothing else. Two
/// variants, so two arms — the compiler counts them exactly as it counts ours.
fn release_note(latest: Option<u32>, read: u32) -> String {
    match latest {
        None => "nothing published yet".to_string(),
        Some(n) if n == read => "caught up".to_string(),
        Some(n) if n > read => format!("{} behind", n - read),
        Some(_) => "ahead of the release".to_string(),
    }
}

fn main() {
    let pairs = [
        (
            Progress::Finished { rating: 9 },
            Progress::Finished { rating: 9 },
        ),
        (
            Progress::Finished { rating: 9 },
            Progress::Finished { rating: 4 },
        ),
        (Progress::NotStarted, Progress::NotStarted),
        (
            Progress::Finished { rating: 7 },
            Progress::Reading { chapter: 3 },
        ),
        (
            Progress::Reading { chapter: 12 },
            Progress::Reading { chapter: 40 },
        ),
        (Progress::NotStarted, Progress::Reading { chapter: 1 }),
    ];
    for (mine, theirs) in &pairs {
        println!("{}", compare(mine, theirs));
    }

    let shelf_of = [
        Progress::NotStarted,
        Progress::Reading { chapter: 12 },
        Progress::Finished { rating: 9 },
    ];

    println!();
    for progress in &shelf_of {
        println!("{}", chapters_read(progress));
    }

    println!();
    for progress in &shelf_of {
        println!("{}", tag(progress));
    }

    println!();
    println!("{}", release_note(None, 0));
    println!("{}", release_note(Some(40), 40));
    println!("{}", release_note(Some(43), 40));
    println!("{}", release_note(Some(38), 40));
}
