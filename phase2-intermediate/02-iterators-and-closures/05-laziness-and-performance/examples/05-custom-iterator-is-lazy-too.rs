//! You already know why this is true — you built it yourself in 2.2.4.
//! `Doubling` is a plain struct implementing `Iterator`; it is exactly as
//! lazy as `.map()` is, for the exact same reason: nothing happens until
//! something calls `.next()`.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 05-custom-iterator-is-lazy-too

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
    println!("constructing Doubling over 0..1_000_000...");
    let mut doubling = Doubling {
        inner: 0..1_000_000,
        calls: 0,
    };
    println!("constructed — calls so far: {}", doubling.calls);

    let first_three: Vec<i32> = doubling.by_ref().take(3).collect();
    println!("first_three: {first_three:?}");
    println!("next() actually ran {} times", doubling.calls);
}
