//! DELIBERATELY BROKEN — expected: E0004
//!
//!     cargo run -p p1-05-04-match-in-depth --example 06-a-new-variant --features broken
//!
//! This is the payoff of the whole module. `Paused` was added to the enum and
//! nothing else was touched. Count the errors, then count the functions: three
//! of the four are errors and the fourth is silent. The silent one is the one
//! to be frightened of.

enum Progress {
    NotStarted,
    Reading { chapter: u32 },
    Finished { rating: u8 },
    Dropped { at: u32 },
    Paused { at: u32 },
}

fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter } => format!("chapter {chapter}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at } => format!("dropped at chapter {at}"),
    }
}

fn is_open(progress: &Progress) -> bool {
    match progress {
        Progress::NotStarted => true,
        Progress::Reading { .. } => true,
        Progress::Finished { .. } => false,
        Progress::Dropped { .. } => false,
    }
}

fn chapters_read(progress: &Progress) -> u32 {
    match progress {
        Progress::NotStarted => 0,
        Progress::Reading { chapter } => *chapter,
        Progress::Finished { .. } => 0,
        Progress::Dropped { at } => *at,
    }
}

// And the fourth one, which the compiler says nothing about. `_` promised to
// handle everything else, so `Paused` quietly became `false` and nobody was
// asked whether that was right.
fn is_finished(progress: &Progress) -> bool {
    match progress {
        Progress::Finished { .. } => true,
        _ => false,
    }
}

fn main() {
    let paused = Progress::Paused { at: 17 };
    println!("{}", describe(&paused));
    println!("{}", is_open(&paused));
    println!("{}", chapters_read(&paused));
    println!("{}", is_finished(&paused));
}
