//! DELIBERATELY BROKEN — expected: a run-time panic, "RefCell already
//! borrowed". It compiles cleanly — `borrow_mut()` type-checks no matter how
//! many other borrows are already alive — and then it dies when you run it.
//!
//!     cargo run -p p2-06-05-refcell-and-interior-mutability --example 07-double-borrow-mut-panics --features broken

use std::cell::RefCell;

fn main() {
    let hype = RefCell::new(0);
    let _first = hype.borrow_mut(); // RefMut<i32>, alive until the end of main
    let _second = hype.borrow_mut(); // still alive — this panics
    println!("unreachable");
}
