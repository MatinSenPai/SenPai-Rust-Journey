//! Shadowing: reusing a name for a genuinely new binding.
//!
//!     cargo run -p p1-01-01-variables-mutability-shadowing --example 02-shadowing

fn main() {
    let total = 100;
    println!("start:      {total}");

    // A brand-new `total`, computed from the old one. The old one still
    // exists — it's just no longer reachable by that name.
    let total = total * 2;
    println!("doubled:    {total}");

    let total = total - 30;
    println!("less 30:    {total}");

    // Shadowing can change the type. `mut` never can.
    let total = total.to_string();
    println!("as text:    {total}");

    // An inner block gets its own scope, so its shadow ends with the block.
    let level = 1;
    {
        let level = 99;
        println!("inside:     {level}");
    }
    println!("outside:    {level}");
}
