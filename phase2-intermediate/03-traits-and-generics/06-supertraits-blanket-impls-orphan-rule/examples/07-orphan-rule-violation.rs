//! DELIBERATELY BROKEN — expected: E0117
//!
//!     cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 07-orphan-rule-violation --features broken

use std::fmt;

// Both foreign: `Display` is std's trait, `Vec` is std's type. Neither side
// is local to this crate, so the orphan rule refuses this `impl`.
impl fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} numbers", self.len())
    }
}

fn main() {
    let nums = vec![1, 2, 3];
    println!("{nums}");
}
