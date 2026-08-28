//! DELIBERATELY BROKEN — expected: E0277
//!
//! `Shelf`'s bound lives on the *whole* impl block this time, not on the one
//! method that actually needs it. `new` never compares anything — but it is
//! stuck in that block too, so a `T` that is not `PartialOrd` cannot even be
//! used to build an empty `Shelf`.
//!
//!     cargo run -p p2-03-02-generic-functions-and-structs --example 05-impl-block-bound --features broken

struct Shelf<T> {
    items: Vec<T>,
}

impl<T: PartialOrd> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }
}

struct MangaVolume {
    title: String,
}

fn main() {
    let _shelf: Shelf<MangaVolume> = Shelf::new();
}
