//! DELIBERATELY BROKEN — expected: E0004
//!
//!     cargo run -p p1-05-04-match-in-depth --example 07-guards-do-not-count --features broken
//!
//! Between them these two arms cover every `u8` there is. The compiler still
//! refuses, and the reason is worth knowing: it does not evaluate guards.

fn shelf(stars: u8) -> String {
    match stars {
        s if s >= 8 => format!("favourites ({s}/10)"),
        s if s < 8 => format!("the rest ({s}/10)"),
    }
}

fn main() {
    println!("{}", shelf(9));
    println!("{}", shelf(2));
}
