//! Three signatures, three different demands on the caller.
//!
//!     cargo run -p p1-02-04-ownership-across-functions --example 03-what-the-signature-says

fn main() {
    let owned = String::from("hello");

    // Borrowing: the caller keeps the value. A literal works too, because a
    // literal is already a `&str`.
    println!("by_view:    {}", by_view(&owned));
    println!("by_view:    {}", by_view("a literal"));
    println!("still ours: {owned}");

    // Taking ownership: the caller gives it up. That is a real demand, and it
    // should be in the signature only when the function genuinely needs it.
    let consumed = String::from("goodbye");
    println!("by_value:   {}", by_value(consumed));
    // `consumed` is gone.

    // Taking ownership when you only needed to read forces the caller to
    // clone, and now there is an allocation that buys nothing.
    println!("wasteful:   {}", by_value(owned.clone()));
    println!("still ours: {owned}");

    println!();
    println!("&str      -> I only want to look; keep it");
    println!("String    -> I need to keep it, or change it, or store it");
    println!("&String   -> almost never: it is &str with extra restrictions");
}

/// The default choice for reading text.
fn by_view(text: &str) -> usize {
    text.len()
}

/// Right when the function stores the value, or returns a changed version of
/// it, or genuinely needs to be its owner.
fn by_value(mut text: String) -> String {
    text.push('!');
    text
}
