//! `HashMap` promises O(1)-average lookup and insert — and nothing at all
//! about the order you get back when you walk it. Not insertion order, not
//! alphabetical order, not even the same order from one run of this exact
//! program to the next.
//!
//! Run this twice in a row and compare:
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 02-no-guaranteed-order

use std::collections::HashMap;

fn main() {
    let mut ranks: HashMap<&str, u32> = HashMap::new();
    ranks.insert("zeta", 1);
    ranks.insert("alpha", 2);
    ranks.insert("mu", 3);
    ranks.insert("beta", 4);
    ranks.insert("quill", 5);
    ranks.insert("delta", 6);

    println!("inserted in this order: zeta, alpha, mu, beta, quill, delta");

    // Two walks, same run, nothing mutated in between: the order matches
    // itself. That is a property of one running program, not a promise
    // about the type.
    print!("walk one:   ");
    for (name, _) in &ranks {
        print!("{name} ");
    }
    println!();

    print!("walk two:   ");
    for (name, _) in &ranks {
        print!("{name} ");
    }
    println!();
}
