//! `Vec<i32>` and `Display` are both foreign, so `impl Display for Vec<i32>`
//! directly is illegal (see `07-orphan-rule-violation.rs`). Wrapping the
//! `Vec` in a local tuple struct makes the outer type local — same trick
//! 1.5.2 used for type safety, now used to satisfy the orphan rule.

use std::fmt;

struct Numbers(Vec<i32>);

impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: i32 = self.0.iter().sum();
        write!(f, "{} numbers, sum {}", self.0.len(), total)
    }
}

fn main() {
    let nums = Numbers(vec![1, 2, 3, 4]);
    println!("{nums}");
}
