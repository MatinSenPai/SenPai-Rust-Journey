//! DELIBERATELY BROKEN — expected: E0599.
//!
//! `HashMap` needs `Hash` and `Eq` on its key type to place and find entries
//! at all. `Coord` here derives neither, so `.insert()` is not callable on
//! this particular `HashMap<Coord, char>` — its trait bounds are not met.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 08-key-missing-hash-eq --features broken

use std::collections::HashMap;

#[derive(Debug)]
struct Coord {
    row: i32,
    col: i32,
}

fn main() {
    let mut board: HashMap<Coord, char> = HashMap::new();
    board.insert(Coord { row: 0, col: 0 }, 'X');
}
