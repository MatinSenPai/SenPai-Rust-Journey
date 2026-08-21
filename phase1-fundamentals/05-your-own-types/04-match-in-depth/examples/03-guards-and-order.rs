//! Arms are tried top to bottom and the first one that matches wins.
//!
//! **This example compiles with a warning on purpose.** Read it — it is the
//! compiler telling you that one arm can never run.
//!
//!     cargo run -p p1-05-04-match-in-depth --example 03-guards-and-order

/// `if n > 20` is a **guard**: an extra condition on an arm that has already
/// matched. A pattern alone cannot express "these two numbers are related".
fn queue_note(chapter: u32, unread: u32) -> String {
    match (chapter, unread) {
        (_, 0) => "all caught up".to_string(),
        (0, n) => format!("never opened, {n} waiting"),
        (c, n) if n > 20 => format!("chapter {c}, {n} behind — good luck"),
        (c, n) => format!("chapter {c}, {n} to go"),
    }
}

/// The same arms in a different order. `(_, 0)` moved down, so the general
/// arms above it swallow the case it was written for.
fn queue_note_reordered(chapter: u32, unread: u32) -> String {
    match (chapter, unread) {
        (0, n) => format!("never opened, {n} waiting"),
        (c, n) if n > 20 => format!("chapter {c}, {n} behind — good luck"),
        (c, n) => format!("chapter {c}, {n} to go"),
        (_, 0) => "all caught up".to_string(),
    }
}

/// A guard can compare two values bound by the same pattern — which no pattern
/// on its own can do.
fn is_at_the_end(chapter: u32, of: u32) -> String {
    match (chapter, of) {
        (c, total) if c == total => "waiting for the next chapter".to_string(),
        (c, total) => format!("{} chapters left", total - c),
    }
}

fn main() {
    println!("{}", queue_note(0, 0));
    println!("{}", queue_note(0, 7));
    println!("{}", queue_note(12, 31));
    println!("{}", queue_note(12, 3));

    println!();
    // Same inputs, the reordered version. Look at the first line.
    println!("{}", queue_note_reordered(0, 0));
    println!("{}", queue_note_reordered(0, 7));
    println!("{}", queue_note_reordered(12, 31));
    println!("{}", queue_note_reordered(12, 3));

    println!();
    println!("{}", is_at_the_end(40, 40));
    println!("{}", is_at_the_end(12, 40));
}
