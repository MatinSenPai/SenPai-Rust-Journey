//! The same job the broken example 04 tries to do, written three ways the
//! borrow checker accepts. None of them fight the rule; each one arranges the
//! code so the rule is not in the way.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 03-restructuring

fn main() {
    // Fix 1 — two passes. Read through a shared borrow and write down what
    // you want to add; then let that borrow finish and do the adding.
    let mut names = vec![String::from("Matin"), String::from("Sara")];
    let mut additions = Vec::new();
    for name in &names {
        let mut polite = name.clone();
        polite.push_str("-san");
        additions.push(polite);
    }
    for polite in additions {
        names.push(polite);
    }
    println!("two passes: {names:?}");

    // Fix 2 — walk by index. An index is a number, not a borrow, so nothing
    // stays borrowed between one turn of the loop and the next. The length is
    // read once, up front, or the loop would chase its own tail.
    let mut scores = vec![10, 20, 30];
    let original = scores.len();
    for index in 0..original {
        let doubled = scores[index] * 2;
        scores.push(doubled);
    }
    println!("by index:   {scores:?}");

    // Fix 3 — copy the value out. `&scores[0]` is a borrow, and it stays
    // alive as long as you keep it. `scores[0]` on a Copy type is a number
    // that belongs to nobody, and the borrow that produced it is over.
    let front = scores[0];
    scores.push(front);
    println!("copied out: {scores:?}");
}
