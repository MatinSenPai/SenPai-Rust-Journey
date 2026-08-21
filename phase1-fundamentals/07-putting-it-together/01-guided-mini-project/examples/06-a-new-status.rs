//! DELIBERATELY BROKEN — expected: E0004
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 06-a-new-status --features broken
//!
//! Three months later the product wants an "on hold" state, between
//! `Watching` and `Dropped`. One variant is added below and nothing else
//! is touched — watch the compiler produce the complete list of places
//! that now have to catch up. This is 1.5.4's exhaustiveness payoff, on
//! the type this lesson built.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Watching { episode: u32 },
    Finished { rating: u8 },
    Planned,
    Dropped { at: u32 },
    OnHold { since: u32 },
}

struct Entry {
    title: String,
    status: Status,
}

fn describe(entry: &Entry) -> String {
    let detail = match entry.status {
        Status::Watching { episode } => format!("watching, episode {episode}"),
        Status::Finished { rating } => format!("finished, {rating}/10"),
        Status::Planned => "planned".to_string(),
        Status::Dropped { at } => format!("dropped at episode {at}"),
    };
    format!("{} — {detail}", entry.title)
}

fn status_tag(status: &Status) -> &str {
    match status {
        Status::Watching { .. } => "watching",
        Status::Finished { .. } => "finished",
        Status::Planned => "planned",
        Status::Dropped { .. } => "dropped",
    }
}

fn main() {
    let entry = Entry {
        title: String::from("Frieren"),
        status: Status::OnHold { since: 5 },
    };
    println!("{}", describe(&entry));
    println!("{}", status_tag(&entry.status));
}
