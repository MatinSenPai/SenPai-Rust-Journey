//! `Fibonacci` only ever defines `next()`. Every method below — `.take()`,
//! `.map()`, `.filter()`, `.enumerate()`, `.collect()`, `.sum()` — is a
//! default method the standard library already wrote on top of `next()`.
//! Nothing here was written for `Fibonacci` specifically.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 03-fibonacci-adapters-and-consumers-free

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

    fn next(&mut self) -> Option<u64> {
        let value = self.current;
        let new_next = self.current + self.next;
        self.current = self.next;
        self.next = new_next;
        Some(value)
    }
}

fn main() {
    let first_ten: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("first 10:      {first_ten:?}");

    let even_only: Vec<u64> = Fibonacci::new().take(10).filter(|n| n % 2 == 0).collect();
    println!("even, of those: {even_only:?}");

    let doubled: Vec<u64> = Fibonacci::new().take(5).map(|n| n * 2).collect();
    println!("first 5, doubled: {doubled:?}");

    let sum: u64 = Fibonacci::new().take(10).sum();
    println!("sum of first 10: {sum}");

    for (index, value) in Fibonacci::new().take(4).enumerate() {
        println!("index {index} -> {value}");
    }
}
