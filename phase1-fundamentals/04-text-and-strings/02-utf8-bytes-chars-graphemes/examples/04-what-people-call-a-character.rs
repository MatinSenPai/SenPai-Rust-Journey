//! Where `.chars().count()` stops being "how many characters" — combining
//! marks, joiners, and two spellings of the same picture.
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 04-what-people-call-a-character

fn main() {
    println!("bytes  chars  seen   what it is                    text");
    show("a plain Persian word", "سلام", 4);
    show("the same word with a fatha", "سَلام", 4);
    show("a word with a ZWNJ in it", "می‌روم", 5);
    show("e plus a combining acute", "e\u{301}", 1);
    show("the precomposed e-acute", "é", 1);
    show("the flag of Iran", "🇮🇷", 1);
    show("a family emoji", "👨‍👩‍👧", 1);

    // Two spellings that render identically and are not equal.
    let decomposed = "e\u{301}";
    let precomposed = "é";
    println!();
    println!("decomposed  {decomposed}  = {} bytes", decomposed.len());
    println!("precomposed {precomposed}  = {} bytes", precomposed.len());
    println!("decomposed == precomposed: {}", decomposed == precomposed);

    // The Persian version of the same trap: لا written as two letters, and
    // the single-scalar ligature that looks the same.
    let two_letters = "لا";
    let ligature = "\u{fefb}";
    println!();
    println!(
        "two letters  {} bytes / {} chars   {two_letters}",
        two_letters.len(),
        two_letters.chars().count()
    );
    println!(
        "one ligature {} bytes / {} chars   {ligature}",
        ligature.len(),
        ligature.chars().count()
    );
    println!("two_letters == ligature: {}", two_letters == ligature);

    // And the one that has nothing to do with counting at all.
    let persian_spelling = "\u{06a9}\u{06cc}\u{0627}\u{0646}"; // keheh + farsi yeh
    let arabic_spelling = "\u{0643}\u{064a}\u{0627}\u{0646}"; // kaf + arabic yeh
    println!();
    println!(
        "Persian keys: {persian_spelling}   {} bytes",
        persian_spelling.len()
    );
    println!(
        "Arabic  keys: {arabic_spelling}   {} bytes",
        arabic_spelling.len()
    );
    println!("equal? {}", persian_spelling == arabic_spelling);
}

/// Byte count, scalar count, the number a person counts by eye, and the text
/// itself last.
fn show(label: &str, text: &str, seen: usize) {
    println!(
        "{:>5}  {:>5}  {seen:>4}   {label:<28}  {text}",
        text.len(),
        text.chars().count()
    );
}
