//! Four ways to walk the same text: `.as_bytes()`, `.bytes()`, `.chars()`
//! and `.char_indices()`.
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 03-walking-the-text

fn main() {
    walk("سلام");
    println!();
    walk("من 🌸");
}

fn walk(text: &str) {
    println!("text: {text}");
    println!("  len()           = {} bytes", text.len());
    println!("  chars().count() = {} scalars", text.chars().count());
    println!();

    // `.as_bytes()` hands over the whole buffer at once, as a `&[u8]`.
    print!("  as_bytes() ->");
    for byte in text.as_bytes() {
        print!(" {byte:02X}");
    }
    println!();

    // `.bytes()` walks the very same bytes one at a time.
    let mut walked = 0;
    for byte in text.bytes() {
        print!("  byte {walked} = {byte:02X}");
        walked += 1;
    }
    println!();
    println!("  bytes() walked {walked} of them");
    println!();

    // `.chars()` decodes those bytes back into scalars.
    // `.char_indices()` does the same and tells you where each one started.
    println!("  char_indices():");
    for (at, letter) in text.char_indices() {
        let width = letter.len_utf8();
        println!("    starts at byte {at}, {width} byte(s) wide     {letter}");
    }
    println!();

    // Which byte offsets are the *start* of a scalar, and which are not.
    print!("  is_char_boundary:");
    for at in 0..=text.len() {
        print!(" {at}={}", text.is_char_boundary(at));
    }
    println!();
}
