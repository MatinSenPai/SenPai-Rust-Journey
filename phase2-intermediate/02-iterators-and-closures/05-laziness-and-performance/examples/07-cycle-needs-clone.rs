//! DELIBERATELY BROKEN — expected: E0277 (a second, cascading E0599 follows
//! it — see "Errors you will meet").
//!
//! `.cycle()` has to restart its source from the beginning every time it
//! runs out, which means it needs a way to make a fresh copy of the
//! original state — it requires `Self: Clone`. `Doubling` (the same struct
//! from 2.2.4-style code) never derived `Clone`, so this does not compile.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 07-cycle-needs-clone --features broken

struct Doubling {
    inner: std::ops::Range<i32>,
    calls: u32,
}

impl Iterator for Doubling {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.calls += 1;
        self.inner.next().map(|n| n * 2)
    }
}

fn main() {
    let doubling = Doubling {
        inner: 0..3,
        calls: 0,
    };
    let cycled: Vec<i32> = doubling.cycle().take(5).collect();
    println!("{cycled:?}");
}
