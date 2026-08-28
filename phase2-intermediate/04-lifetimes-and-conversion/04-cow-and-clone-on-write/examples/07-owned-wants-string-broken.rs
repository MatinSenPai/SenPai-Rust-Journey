//! DELIBERATELY BROKEN — expected: E0271
//!
//! `Cow::Owned` holds `<B as ToOwned>::Owned`, not `B` itself. For
//! `Cow<'_, str>` that is `String` — a bare `&str` does not fit, even
//! though it looks "owned enough."
//!
//!     cargo run -p p2-04-04-cow-and-clone-on-write --example 07-owned-wants-string-broken --features broken

use std::borrow::Cow;

fn main() {
    let value: Cow<str> = Cow::Owned("Trigun");
    println!("{value}");
}
