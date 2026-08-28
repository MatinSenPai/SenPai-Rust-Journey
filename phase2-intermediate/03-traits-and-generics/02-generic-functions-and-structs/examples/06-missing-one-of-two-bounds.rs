//! DELIBERATELY BROKEN — expected: E0599
//!
//! `announce` with the `Clone` bound removed — only `Display` is left.
//! Formatting `item` still works, but `.clone()` needs a bound of its own;
//! `Display` never promised that.
//!
//!     cargo run -p p2-03-02-generic-functions-and-structs --example 06-missing-one-of-two-bounds --features broken

use std::fmt::Display;

fn announce<T: Display>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}

fn main() {
    let (headline, _kept) = announce(String::from("Frieren"));
    println!("{headline}");
}
