//! Borrowing is fine when the source outlives its excerpts. Owning is
//! what you reach for when it can't.
//!
//! Run:
//! `cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 05-borrow-or-own`

struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    fn first_line(source: &'a str) -> Self {
        let text = source.lines().next().unwrap_or(source);
        Excerpt { text }
    }
}

/// Borrowing works here: `logs` outlives every `Excerpt` this returns.
fn first_lines_of<'a>(logs: &'a [String]) -> Vec<Excerpt<'a>> {
    logs.iter().map(|log| Excerpt::first_line(log)).collect()
}

/// A fresh `String`, as if just downloaded — gone at the end of this call.
fn fetch_response(id: u32) -> String {
    format!("200 OK (request {id})\nbody omitted")
}

/// Owning is what you need here: each `response` is temporary, built fresh
/// on every loop iteration, so no `Excerpt<'a>` could borrow from it and
/// still be around by the time this function returns.
fn first_line_of_each_response(request_ids: &[u32]) -> Vec<String> {
    let mut summaries = Vec::new();
    for id in request_ids {
        let response = fetch_response(*id);
        let first_line = response.lines().next().unwrap_or("").to_string();
        summaries.push(first_line);
    }
    summaries
}

fn main() {
    let logs = vec![
        String::from("started\nrunning"),
        String::from("connected\nidle"),
    ];
    for excerpt in first_lines_of(&logs) {
        println!("borrowed: {}", excerpt.text);
    }

    for summary in first_line_of_each_response(&[1, 2]) {
        println!("owned:    {summary}");
    }
}
