//! DELIBERATELY BROKEN — expected: E0425
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 07-binding-escaped --features broken
//!
//! An `if let` binding lives inside its block. A `let ... else` binding does
//! not — and that is the whole difference between the two.

fn main() {
    let rating: Option<u8> = Some(9);

    if let Some(score) = rating {
        println!("inside:  {score}");
    }

    println!("outside: {score}");
}
