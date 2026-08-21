//! `let` versus `let mut`, and what each one lets you do.
//!
//!     cargo run -p p1-01-01-variables-mutability-shadowing --example 01-immutable

fn main() {
    // Immutable: bound once, and that's what it is everywhere below.
    let orders = 7;
    println!("orders: {orders}");

    // Mutable: you opted in, so it can be updated in place.
    let mut remaining = 7;
    println!("remaining: {remaining}");
    remaining -= 3;
    println!("after shipping three: {remaining}");

    // A constant. Always immutable, type always written out, and computed
    // before the program ever runs.
    const MAX_PER_ORDER: u32 = 50;
    println!("max per order: {MAX_PER_ORDER}");
}
