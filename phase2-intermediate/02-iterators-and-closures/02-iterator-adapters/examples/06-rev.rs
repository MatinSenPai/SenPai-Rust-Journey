//! `.rev()` walks the same items back to front. It only exists on iterators
//! that implement `DoubleEndedIterator` — an iterator that can also answer
//! `next_back()`, "give me the next item counting from the back."
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 06-rev

fn main() {
    let recently_added = vec!["Frieren", "Bocchi the Rock!", "Mushoku Tensei"];

    // A slice's iterator can pull from both ends of the same walk-through —
    // that is what "double-ended" means, literally.
    let mut both_ends = recently_added.iter();
    println!("next():      {:?}", both_ends.next());
    println!("next_back(): {:?}", both_ends.next_back());
    println!("next():      {:?}", both_ends.next());
    println!("next():      {:?}", both_ends.next());

    println!();

    // `.rev()` is that same idea, wrapped up as an adapter.
    for title in recently_added.iter().rev() {
        println!("newest first: {title}");
    }

    println!();

    // It composes with everything else — it is just another adapter.
    for (index, title) in recently_added.iter().rev().enumerate() {
        println!("{index} back from the end: {title}");
    }
}
