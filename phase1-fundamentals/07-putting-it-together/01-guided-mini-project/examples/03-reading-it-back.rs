//! Stage 3 — reading it back. `match` on `Status`, covering every variant,
//! binding each one's data in the same step that recognises it.
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 03-reading-it-back

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Watching { episode: u32 },
    Finished { rating: u8 },
    Planned,
    Dropped { at: u32 },
}

struct Entry {
    title: String,
    status: Status,
}

// Four variants in, four arms out. No `_`: naming every one of them is what
// makes this exhaustive, and exhaustive is what 06-a-new-status.rs (behind
// `--features broken`) spends its whole file proving.
fn describe(entry: &Entry) -> String {
    let detail = match entry.status {
        Status::Watching { episode } => format!("watching, episode {episode}"),
        Status::Finished { rating } => format!("finished, {rating}/10"),
        Status::Planned => "planned".to_string(),
        Status::Dropped { at } => format!("dropped at episode {at}"),
    };
    format!("{} — {detail}", entry.title)
}

fn main() {
    let library = vec![
        Entry {
            title: String::from("Cowboy Bebop"),
            status: Status::Watching { episode: 9 },
        },
        Entry {
            title: String::from("Frieren"),
            status: Status::Finished { rating: 9 },
        },
        Entry {
            title: String::from("Bocchi the Rock!"),
            status: Status::Planned,
        },
        Entry {
            title: String::from("حمله به تایتان"),
            status: Status::Dropped { at: 12 },
        },
    ];

    for entry in &library {
        println!("{}", describe(entry));
    }
}
