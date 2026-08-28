//! `as` never fails on an integer cast — it just keeps the low bits and
//! throws the rest away. `try_into()` is the same conversion `try_from`
//! gives you, read from the destination's side, exactly like `.into()`
//! mirrors `from`.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 03-as-vs-try-into

fn main() {
    let big: i32 = 300;

    let truncated = big as u8;
    println!("300i32 as u8:             {truncated}");

    let honest: Result<u8, _> = big.try_into();
    println!("300i32.try_into():        {honest:?}");

    // 256 wraps all the way back around to 0 — the low 8 bits of 256 are
    // all zero.
    let wrapped = 256i32 as u8;
    println!("256i32 as u8:             {wrapped}");
}
