//! DELIBERATELY BROKEN — expected: E0502
//!
//! `.to_mut()` takes `&mut self`. Holding on to what it returns keeps that
//! mutable borrow alive — so reading `value` immutably while still holding
//! it is the same aliasing violation as any other `&mut` next to a `&`.
//!
//!     cargo run -p p2-04-04-cow-and-clone-on-write --example 08-to-mut-borrow-conflict-broken --features broken

use std::borrow::Cow;

fn main() {
    let mut value: Cow<str> = Cow::Borrowed("Trigun");
    let episodes = value.to_mut();
    println!("before: {value}");
    episodes.push_str(" - 26 episodes");
}
