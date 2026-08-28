//! Run: cargo run -p p2-04-01-lifetime-basics-and-elision --example 02-elision-rule-two
//!
//! One reference in, one reference out — `first_word` and `first_word_explicit`
//! are the exact same function to the compiler. Elision rule 2 inserts the
//! second definition's lifetimes for you whenever the first is what you type.

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn first_word_explicit<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

fn main() {
    let sentence = "senpai teaches rust";
    println!("elided:   {}", first_word(sentence));
    println!("explicit: {}", first_word_explicit(sentence));
}
