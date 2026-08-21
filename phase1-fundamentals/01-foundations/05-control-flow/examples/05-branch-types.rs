//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-01-05-control-flow --example 05-branch-types --features broken

fn main() {
    let score = 73;

    // A binding has exactly one type, so every branch has to agree on it.
    let grade = if score >= 70 { 'B' } else { "failed" };

    println!("{grade}");
}
