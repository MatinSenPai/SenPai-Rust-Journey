//! DELIBERATELY BROKEN — expected: E0133
//!
//! Creating `ptr` needed no `unsafe` at all. Reading through it does — and
//! this line reads through it outside any `unsafe` block.
//!
//!     cargo run -p p2-10-04-unsafe-for-real --example 04-deref-outside-unsafe-broken --features broken

fn main() {
    let mut value = 10;
    let ptr: *mut i32 = &mut value;
    let doubled = *ptr * 2;
    println!("{doubled}");
}
