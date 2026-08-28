//! One scalar takes one to four bytes inside a `String` — and always four
//! inside a `char`.
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 01-one-scalar-many-bytes

fn main() {
    // A `char` holds exactly one Unicode scalar value, and it is four bytes
    // wide whatever it holds: room for the largest scalar there is.
    println!("a char in memory: {} bytes, always", size_of::<char>());
    println!();

    println!("scalar    utf-8    name                                it looks like");
    show('a', "LATIN SMALL LETTER A");
    show('س', "ARABIC LETTER SEEN");
    show('م', "ARABIC LETTER MEEM");
    show('۵', "EXTENDED ARABIC-INDIC DIGIT FIVE");
    show('\u{200c}', "ZERO WIDTH NON-JOINER");
    show('\u{064e}', "ARABIC FATHA");
    show('€', "EURO SIGN");
    show('🌸', "CHERRY BLOSSOM");

    println!();
    println!("and the bytes UTF-8 actually stores:");
    bytes_of('a');
    bytes_of('س');
    bytes_of('€');
    bytes_of('🌸');
}

/// One row: the scalar's number, its encoded width, its official name, and the
/// character itself *last* — so the Persian does not drag the columns around.
fn show(letter: char, name: &str) {
    let width = letter.len_utf8();
    let unit = if width == 1 { "byte " } else { "bytes" };
    let pad = if letter as u32 > 0xFFFF { "" } else { " " };
    println!(
        "U+{:04X}{pad}   {width} {unit}  {name:<34}  {letter}",
        letter as u32
    );
}

/// The same scalar again, this time as the bytes it becomes in a `String`.
fn bytes_of(letter: char) {
    let mut buffer = [0u8; 4];
    let encoded = letter.encode_utf8(&mut buffer);
    print!("  U+{:04X} ->", letter as u32);
    for byte in encoded.as_bytes() {
        print!("  {byte:02X} ({byte:08b})");
    }
    println!();
}
