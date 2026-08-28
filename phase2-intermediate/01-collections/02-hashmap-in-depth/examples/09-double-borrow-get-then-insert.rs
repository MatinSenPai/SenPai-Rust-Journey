//! DELIBERATELY BROKEN — expected: E0502.
//!
//! `current` borrows `scores` immutably, and its last use — the `println!`
//! below — comes *after* the `.insert()` call that needs to borrow `scores`
//! mutably. Two live borrows, one of them mutable: the alias rule from 1.3.2
//! does not stop applying just because the value now lives in a `HashMap`.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 09-double-borrow-get-then-insert --features broken

use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("alice".to_string(), 10);

    if let Some(current) = scores.get("alice") {
        scores.insert("bob".to_string(), *current);
        println!("alice's score is still {current}");
    }
}
