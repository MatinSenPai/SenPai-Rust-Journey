//! DELIBERATELY BROKEN — expected: E0502
//! Run `cargo run -p p1-03-03-borrow-scopes-and-nll \
//!   --example 09-a-borrow-you-named-yourself --features broken`.
//!
//! `items.push(items.len())` compiles — see examples/04-two-phase-borrows.rs.
//! Writing the same `&mut` out by hand and giving it a name does not.

fn main() {
    let mut items = vec![10, 20, 30];

    let handle = &mut items;
    handle.push(items.len());

    println!("{handle:?}");
}
