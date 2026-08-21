//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 07-indexing-by-number --features broken
//!
//! The second thing everybody does after their first boundary panic: give up
//! on ranges and reach for a single index instead.

fn main() {
    let persian = "برنامه‌نویسی";

    let third = persian[2];

    println!("{third}");
}
