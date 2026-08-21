//! Stage 4 — text done properly. A summary line built with `format!`, a
//! column of titles that stays aligned, and a truncation that counts
//! *characters* — the only version that is safe on Persian input.
//!
//!     cargo run -p p1-07-01-guided-mini-project --example 04-text-done-properly

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

// Straight out of 1.4.4: the returned index came from `char_indices` itself,
// so it is always a char boundary, and the slice can never panic — not on
// "Cowboy Bebop", not on "حمله به تایتان".
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}

// An ellipsis only when something was actually cut — also 1.4.4. The test
// is `.chars().count()`, not `.len()`, or a Persian title half the length
// of an English one would get a lying ellipsis it never earned.
fn short_title(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        format!("{}…", truncate_to_chars(text, max_chars))
    }
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
    let library = vec![
        Entry {
            title: String::from("Cowboy Bebop"),
            status: Status::Watching { episode: 9 },
        },
        Entry {
            title: String::from("حمله به تایتان"),
            status: Status::Dropped { at: 12 },
        },
        Entry {
            title: String::from("Frieren"),
            status: Status::Finished { rating: 9 },
        },
        Entry {
            title: String::from("Bocchi the Rock!"),
            status: Status::Planned,
        },
    ];

    // Rust's `{:<width$}` pads by character count, not byte count — try it
    // yourself and the Persian row lines up exactly like the English ones.
    // What is unsafe is *cutting* at a byte offset, not padding one.
    const MAX_CHARS: usize = 10;
    for entry in &library {
        let short = short_title(&entry.title, MAX_CHARS);
        println!(
            "{:<width$}  {}",
            short,
            status_tag(&entry.status),
            width = MAX_CHARS + 1
        );
    }

    // Four counters, not a `HashMap<Status, u32>` — the right tool for
    // counting by an open-ended key is in Phase 2's collections module;
    // for four known cases a `Vec` of counters reads just as clearly.
    let mut watching = 0;
    let mut finished = 0;
    let mut planned = 0;
    let mut dropped = 0;
    for entry in &library {
        match entry.status {
            Status::Watching { .. } => watching += 1,
            Status::Finished { .. } => finished += 1,
            Status::Planned => planned += 1,
            Status::Dropped { .. } => dropped += 1,
        }
    }

    println!();
    println!(
        "{} entries — {watching} watching, {finished} finished, {planned} planned, {dropped} dropped",
        library.len()
    );
}
