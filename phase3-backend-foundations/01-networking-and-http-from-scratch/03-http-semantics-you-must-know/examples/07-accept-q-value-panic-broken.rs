//! DELIBERATELY BROKEN — expected: a run-time panic on an unparseable `q`
//! value.
//!
//! A real `Accept` header's `q` is typed by hand by whoever configured the
//! client — trusting it to always parse is exactly the kind of assumption
//! 3.1.2 warned about for request bytes in general.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 07-accept-q-value-panic-broken --features broken

fn weight_of(entry: &str) -> f64 {
    match entry.split_once(";q=") {
        Some((_, weight)) => weight.trim().parse::<f64>().unwrap(),
        None => 1.0,
    }
}

fn main() {
    let entry = "text/html;q=high"; // a human typed a word, not a number
    println!("weight = {}", weight_of(entry));
}
