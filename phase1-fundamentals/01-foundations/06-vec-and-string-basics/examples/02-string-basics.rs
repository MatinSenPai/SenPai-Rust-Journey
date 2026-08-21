//! `String` — the text buffer that grows.
//!
//!     cargo run -p p1-01-06-vec-and-string-basics --example 02-string-basics

fn main() {
    // Three ways to make one, same as Vec.
    let empty = String::new();
    let from_literal = String::from("hello");
    let converted = "hello".to_string();

    println!("empty:     {empty:?}");
    println!("from:      {from_literal:?}");
    println!("converted: {converted:?}");

    // Growing. `push_str` appends text; `push` appends a single char.
    let mut greeting = String::from("hello");
    greeting.push_str(", world");
    greeting.push('!');
    println!("greeting:  {greeting}");

    // And the thing that catches everyone. `len()` counts BYTES.
    let english = String::from("hello");
    let persian = String::from("سلام");

    println!();
    println!("english:   {english}");
    println!("  bytes:   {}", english.len());
    println!("  chars:   {}", english.chars().count());
    println!("persian:   {persian}");
    println!("  bytes:   {}", persian.len());
    println!("  chars:   {}", persian.chars().count());

    // For "hello" the two agree, which is exactly why the difference is easy
    // to miss if all your test data is English. For Persian they do not.

    // A literal is not a String. It is a `&str` — a view of text that is
    // already somewhere, in this case baked into the compiled program.
    let borrowed: &str = "I am a literal";
    let owned: String = borrowed.to_string();
    println!();
    println!("borrowed:  {borrowed}");
    println!("owned:     {owned}");

    // A `String` can always be lent out as a `&str`. That is why functions
    // take `&str` and callers can pass either.
    println!("shout:     {}", shout(&owned));
    println!("shout:     {}", shout("a literal directly"));
}

/// Takes a view, so a caller can pass a `String` or a literal.
fn shout(text: &str) -> String {
    text.to_uppercase()
}
