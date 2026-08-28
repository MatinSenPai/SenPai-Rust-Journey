//! `Iterator` has exactly one method you must supply: `next(&mut self)`.
//! Everything else in this lesson — every adapter, every `for` loop — is
//! built from nothing but repeated calls to it.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 01-next-is-the-whole-trait

fn main() {
    let scores = vec![10, 20, 30];
    let mut by_hand = scores.iter();

    println!("by_hand.next(): {:?}", by_hand.next());
    println!("by_hand.next(): {:?}", by_hand.next());
    println!("by_hand.next(): {:?}", by_hand.next());
    println!("by_hand.next(): {:?}", by_hand.next());
    println!("by_hand.next() again: {:?}", by_hand.next());

    println!();

    // A `for` loop does exactly this, automatically: call `.next()`, bind
    // whatever comes out of the `Some`, run the body, repeat — and stop the
    // moment `next()` answers `None`.
    let mut manual = scores.iter();
    loop {
        match manual.next() {
            Some(value) => println!("loop+match: {value}"),
            None => break,
        }
    }

    println!();

    for value in scores.iter() {
        println!("for loop:   {value}");
    }
}
