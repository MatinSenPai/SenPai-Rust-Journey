//! The reverse of 01: not every i32 fits inside a u8, so that conversion
//! can fail. `TryFrom` is `From`'s fallible sibling — same shape, but
//! `try_from` returns a `Result` instead of handing back `Self` directly.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 02-numeric-narrowing-tryfrom

fn main() {
    let fits: Result<u8, _> = u8::try_from(200i32);
    let overflow: Result<u8, _> = u8::try_from(300i32);
    let negative: Result<u8, _> = u8::try_from(-1i32);

    println!("u8::try_from(200i32):  {fits:?}");
    println!("u8::try_from(300i32):  {overflow:?}");
    println!("u8::try_from(-1i32):   {negative:?}");
}
