//! DELIBERATELY BROKEN — expected: E0308
//!
//!     cargo run -p p1-05-04-match-in-depth --example 08-arms-disagree --features broken
//!
//! A `match` is one expression, so it has one type. Two arms here disagree
//! about what that type is.

fn band(stars: u8) -> String {
    match stars {
        0..=4 => "weak".to_string(),
        5..=7 => "watchable",
        _ => "good".to_string(),
    }
}

fn main() {
    println!("{}", band(2));
    println!("{}", band(6));
}
