//! `HashMap` needs two things from a key type: `Hash` (turn it into a hash
//! code) and `Eq` (tell two keys apart with exact equality, not an
//! approximation). Both are one `#[derive]` away for a plain struct.
//!
//! One sentence on *which* hash: the default algorithm, SipHash, is chosen
//! to resist a hostile caller crafting keys that all collide on purpose —
//! not to be the fastest possible hash. Faster hashers exist for hot paths;
//! that is a swap, not a rewrite, and not this lesson's detour.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 05-custom-key-hash-eq

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    row: i32,
    col: i32,
}

fn main() {
    let mut board: HashMap<Coord, char> = HashMap::new();
    board.insert(Coord { row: 0, col: 0 }, 'X');
    board.insert(Coord { row: 1, col: 2 }, 'O');

    // Two separately-built `Coord`s with the same fields are `==` — that is
    // `PartialEq`/`Eq` — and land in the same bucket — that is `Hash`.
    // Together, that is what lets `.get()` find this key at all.
    let lookup = Coord { row: 1, col: 2 };
    println!("at (1, 2): {:?}", board.get(&lookup));
    println!("at (5, 5): {:?}", board.get(&Coord { row: 5, col: 5 }));
}
