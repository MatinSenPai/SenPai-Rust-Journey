//! Four ways to add two numbers that might not fit.
//!
//!     cargo run -p p1-01-02-scalar-types-and-overflow --example 02-overflow-guards

fn main() {
    let nearly_full: u8 = 250;

    // Returns None instead of producing a wrong answer.
    println!("checked_add(10):     {:?}", nearly_full.checked_add(10));
    println!("checked_add(5):      {:?}", nearly_full.checked_add(5));

    // Clamps at the maximum rather than wrapping round.
    println!("saturating_add(10):  {}", nearly_full.saturating_add(10));

    // Wraps round on purpose — 250 + 10 becomes 4.
    println!("wrapping_add(10):    {}", nearly_full.wrapping_add(10));

    // Both the answer and whether it overflowed.
    println!("overflowing_add(10): {:?}", nearly_full.overflowing_add(10));

    println!();
    println!("Same four exist for sub, mul, div, pow and the rest.");
}
