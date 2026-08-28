//! DELIBERATELY BROKEN — expected: E0004
//!
//!     cargo run -p p1-05-04-match-in-depth --example 05-a-missing-arm --features broken
//!
//! Four variants, three arms. The compiler names the one that is missing.

enum Progress {
    NotStarted,
    Reading { chapter: u32 },
    Finished { rating: u8 },
    Dropped { at: u32 },
}

fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter } => format!("chapter {chapter}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
    }
}

fn main() {
    println!("{}", describe(&Progress::NotStarted));
    println!("{}", describe(&Progress::Dropped { at: 3 }));
}
