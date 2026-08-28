//! What `if let` costs you. `match` is checked for exhaustiveness; `if let`
//! is not. Add a variant and only one of these two says anything.
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 04-the-silent-hole

#[derive(Debug)]
enum Status {
    Watching { episode: u32 },
    Completed { rating: u8 },
    Dropped,
    // Add a fourth variant here — `OnHold`, say — and rebuild.
    //
    //   `describe` stops compiling with E0004 and points at the gap.
    //   `medal`    compiles, runs, and quietly answers "—".
}

fn describe(status: &Status) -> String {
    match status {
        Status::Watching { episode } => format!("on episode {episode}"),
        Status::Completed { rating } => format!("finished, {rating}/10"),
        Status::Dropped => "dropped".to_string(),
    }
}

fn medal(status: &Status) -> String {
    if let Status::Completed { rating } = status {
        format!("{rating}/10")
    } else {
        "—".to_string()
    }
}

fn main() {
    let shelf = [
        Status::Watching { episode: 7 },
        Status::Completed { rating: 9 },
        Status::Dropped,
    ];

    for status in &shelf {
        println!(
            "{:<25} {:<16} {}",
            format!("{status:?}"),
            describe(status),
            medal(status)
        );
    }
}
