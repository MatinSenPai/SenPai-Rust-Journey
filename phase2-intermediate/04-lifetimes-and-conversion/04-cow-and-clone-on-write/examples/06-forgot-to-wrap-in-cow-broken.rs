//! DELIBERATELY BROKEN — expected: E0308
//!
//! `Cow<'_, str>` does not automatically absorb a plain `&str` at a
//! `return` — you have to say which state you mean.
//!
//!     cargo run -p p2-04-04-cow-and-clone-on-write --example 06-forgot-to-wrap-in-cow-broken --features broken

use std::borrow::Cow;

fn passthrough(input: &str) -> Cow<'_, str> {
    input
}

fn main() {
    let value = passthrough("Trigun");
    println!("{value}");
}
