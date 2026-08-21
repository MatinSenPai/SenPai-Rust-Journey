//! DELIBERATELY BROKEN — expected: E0506
//! Run `cargo run -p p1-03-03-borrow-scopes-and-nll \
//!   --example 08-assign-while-borrowed --features broken`.
//!
//! No `&mut` anywhere. A plain assignment is enough.

fn main() {
    let mut level = 5;
    let watcher = &level;

    level = 7;

    println!("watcher saw {watcher}, level is now {level}");
}
