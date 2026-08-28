//! Where the legal cuts are, and how to find them without guessing.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 02-where-the-boundaries-are

fn main() {
    let text = "می‌روم";

    // `.char_indices()` hands you the byte index each character starts at.
    // That list *is* the list of legal cut points, plus `text.len()`.
    println!("{text:?} — {} bytes", text.len());
    for (index, character) in text.char_indices() {
        println!(
            "  {index:>2}..{:<2} {character:?}  U+{:04X}  {} bytes",
            index + character.len_utf8(),
            character as u32,
            character.len_utf8()
        );
    }

    // The same information, asked byte by byte.
    println!();
    print!("boundaries: ");
    for index in 0..=text.len() {
        if text.is_char_boundary(index) {
            print!("{index} ");
        }
    }
    println!();

    // Byte 4 starts a zero-width non-joiner, which is three bytes long. That
    // is the character that makes Persian byte arithmetic unpredictable: most
    // letters are 2 bytes, the joiner between them is 3.
    println!();
    for index in [3usize, 4, 5, 6] {
        println!(
            "index {index}: boundary={:<5} floor={} ceil={}",
            text.is_char_boundary(index),
            text.floor_char_boundary(index),
            text.ceil_char_boundary(index)
        );
    }
}
