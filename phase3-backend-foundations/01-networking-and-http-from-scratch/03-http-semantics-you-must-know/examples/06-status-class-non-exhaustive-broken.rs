//! DELIBERATELY BROKEN — expected: E0004
//!
//! `u16` covers `0..=65535`, but the five HTTP status classes only cover
//! `100..=599`. A `match` that lists exactly those five ranges and nothing
//! else looks complete to a human — it is not exhaustive to `rustc`.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 06-status-class-non-exhaustive-broken --features broken

fn classify(status: u16) -> &'static str {
    match status {
        100..=199 => "Informational",
        200..=299 => "Success",
        300..=399 => "Redirection",
        400..=499 => "ClientError",
        500..=599 => "ServerError",
    }
}

fn main() {
    println!("{}", classify(200));
}
