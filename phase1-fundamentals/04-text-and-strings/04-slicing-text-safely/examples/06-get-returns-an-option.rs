//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 06-get-returns-an-option --features broken
//!
//! The first thing everybody does after their first boundary panic: swap
//! `&text[a..b]` for `text.get(a..b)` and expect the same type back.

fn main() {
    let persian = "برنامه‌نویسی";

    let piece: &str = persian.get(0..6);

    println!("{piece}");
}
