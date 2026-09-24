//! `Accept` is a comma-separated list of media types, each optionally
//! carrying a `;q=WEIGHT`. Parsing that list into (type, weight) pairs is
//! half of content negotiation — *deciding a winner* from them is the
//! other half, and it's this lesson's Build exercise.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 03-parsing-accept-header-pieces

fn parse_accept(accept: &str) -> Vec<(String, f64)> {
    accept
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| match entry.split_once(";q=") {
            Some((media, weight)) => (
                media.trim().to_string(),
                weight.trim().parse().unwrap_or(1.0),
            ),
            None => (entry.to_string(), 1.0),
        })
        .collect()
}

fn main() {
    // A real Accept header Firefox sends on a page navigation (documented on
    // MDN's Accept header page): four entries, three explicit weights, one
    // implicit (`text/html` itself, at the default weight of 1.0).
    let accept = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
    for (media_type, weight) in parse_accept(accept) {
        println!("{media_type:<24} q={weight}");
    }
}
