//! DELIBERATELY BROKEN — expected: E0046.
//!
//! `Iterator` has exactly one required method. Declare the `impl` block and
//! the associated type, but leave `next()` out, and the compiler refuses —
//! there is no default for the one thing every other method is built on.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 06-missing-next-method --features broken

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
}

fn main() {
    let mut fib = Fibonacci::new();
    println!("{:?}", fib.next());
}
