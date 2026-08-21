//! DELIBERATELY BROKEN — expected: a run-time panic, not a compiler error.
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 05-not-a-char-boundary --features broken
//!
//! This one compiles. It is behind the `broken` feature anyway, because
//! running it should be a deliberate act: it crashes the program.

fn main() {
    let word = "سلام";
    println!("the whole word:  {word}");
    println!("its byte length: {}", word.len());

    // "Take the first letter." Written by someone whose letters are one byte
    // each — which is true in English and false here.
    let first = &word[0..1];
    println!("the first letter: {first}");
}
