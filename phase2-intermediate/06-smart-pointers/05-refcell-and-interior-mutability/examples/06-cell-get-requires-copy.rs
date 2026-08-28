//! DELIBERATELY BROKEN — expected: E0599.
//!
//!     cargo run -p p2-06-05-refcell-and-interior-mutability --example 06-cell-get-requires-copy --features broken

use std::cell::Cell;

fn main() {
    let slot: Cell<String> = Cell::new(String::from("Frieren"));
    let title = slot.get();
    println!("{title}");
}
