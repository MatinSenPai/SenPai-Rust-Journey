//! Elision rule 3 doing real work: `text` and `word_count` both take
//! `&self` and return without ever naming `'a`.
//!
//! Run:
//! `cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 04-methods-and-elision`

struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    fn first_line(source: &'a str) -> Self {
        let text = source.lines().next().unwrap_or(source);
        Excerpt { text }
    }

    // No `'a` here. Elision rule 3: one `&self` input, so the elided
    // output lifetime is `self`'s borrow — not the struct's own `'a`.
    fn text(&self) -> &str {
        self.text
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

fn main() {
    let log_entry = String::from("connection reset by peer\nretrying in 3s\ngiving up");
    let excerpt = Excerpt::first_line(&log_entry);

    println!("first line: {}", excerpt.text());
    println!("word count: {}", excerpt.word_count());
}
