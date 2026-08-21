//! DELIBERATELY BROKEN — expected: a run-time panic, "end byte index 7 is
//! not a char boundary". It compiles, and then it dies when you run it.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 04-cut-in-half --features broken

fn main() {
    let english = "programming";
    let persian = "برنامه‌نویسی";

    // Fine. Seven bytes of ASCII is seven letters.
    println!("english: {:?}", &english[0..7]);

    // Not fine. Byte 7 is the second half of a two-byte letter, and Rust
    // refuses to hand you half a character.
    println!("persian: {:?}", &persian[0..7]);
}
