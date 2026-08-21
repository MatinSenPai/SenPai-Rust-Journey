//! The aliasing rule from the side where everything is allowed: any number of
//! shared borrows, or one mutable borrow — one after the other, never mixed.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 01-many-readers-one-writer

fn main() {
    let mut scores = vec![10, 20, 30];

    // Three shared borrows of the same Vec, all alive at once. None of them
    // can write, so none of them can surprise the other two.
    let first = &scores;
    let second = &scores;
    let third = &scores;
    println!("three readers: {first:?} {second:?} {third:?}");

    // Now one exclusive borrow. The readers above are finished with, so this
    // is not "both at once" — it is "one after the other".
    let writer = &mut scores;
    writer.push(40);
    println!("one writer:    {writer:?}");

    // A borrow of a *different* value is never in conflict, however mutable
    // it is. The rule is about one value at a time, not about the program.
    let mut names = vec![String::from("Matin")];
    let other = &mut names;
    other.push(String::from("Sara"));
    println!("another value: {other:?}");
    println!("scores again:  {scores:?}");

    // And a shared borrow is Copy, so handing it around costs nothing and
    // still counts as one more reader.
    let a = &scores;
    let b = a;
    println!("copied arrow:  {a:?} {b:?}");
}
