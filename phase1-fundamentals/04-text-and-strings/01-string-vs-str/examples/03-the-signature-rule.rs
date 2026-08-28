//! One function, every kind of caller — and why the return value is owned.
//!
//!     cargo run -p p1-04-01-string-vs-str --example 03-the-signature-rule

/// Takes a view, because it only reads.
fn byte_length(text: &str) -> usize {
    text.len()
}

/// Returns an owner, because the answer is new text that has to live
/// somewhere after this function ends.
fn shout(text: &str) -> String {
    text.to_uppercase()
}

fn main() {
    let owned = String::from("Matin");
    let literal = "senpai";
    let borrowed: &str = owned.as_str();

    // Three kinds of caller, one signature, and not one byte copied to call
    // it. The middle one is a `&String` — the compiler turned it into a
    // `&str` on the spot.
    println!("literal    -> {}", byte_length(literal));
    println!("&String    -> {}", byte_length(&owned));
    println!("&str       -> {}", byte_length(borrowed));
    println!();

    println!("shout(literal) = {}", shout(literal));
    println!("shout(&owned)  = {}", shout(&owned));
    println!();

    // Persian: uppercasing does nothing to it, and `len()` is still bytes.
    // Which byte belongs to which letter is the next lesson's subject.
    let persian = String::from("سلام دنیا");
    println!("persian        = {persian}");
    println!("shout(persian) = {}", shout(&persian));
    println!("byte_length    = {}", byte_length(&persian));
    println!();

    // `owned` is still ours: nothing here ever took it away.
    println!("still ours: {owned}");
}
