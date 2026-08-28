//! DELIBERATELY BROKEN — expected: E0277
//!
//! `ToOwned::Owned` does not have to be `Self` — which is exactly why it
//! exists. `str` cannot be `Self` here: a return type must have a size known
//! at compile time, and `str` (unlike `String`) never does.
//!
//!     cargo run -p p2-04-03-deref-asref-borrow --example 10-toowned-unsized-return-broken --features broken

fn widen(s: &str) -> str {
    s.to_owned()
}

fn main() {}
