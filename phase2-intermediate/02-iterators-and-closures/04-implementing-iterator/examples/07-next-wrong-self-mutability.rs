//! DELIBERATELY BROKEN — expected: E0053.
//!
//! `next()`'s state advance needs `&mut self`. Writing `&self` here — a very
//! typeable slip — does not match the trait's own signature, and the
//! compiler catches it, not a test.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 07-next-wrong-self-mutability --features broken

struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci {
            current: 0,
            next: 1,
        }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&self) -> Option<u64> {
        Some(self.current)
    }
}

fn main() {
    let mut fib = Fibonacci::new();
    println!("{:?}", fib.next());
}
