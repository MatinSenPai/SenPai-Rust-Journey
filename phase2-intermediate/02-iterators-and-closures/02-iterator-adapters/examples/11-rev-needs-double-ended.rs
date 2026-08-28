//! DELIBERATELY BROKEN — expected: E0277
//!
//! `HashMap` has no defined front or back — its iteration order isn't even
//! specified — so its iterator never implements `DoubleEndedIterator`, and
//! `.rev()` has nothing to call.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 11-rev-needs-double-ended --features broken

use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<&str, u32> = HashMap::new();
    scores.insert("Frieren", 28);
    scores.insert("AOT", 87);

    let _reversed = scores.iter().rev();
}
