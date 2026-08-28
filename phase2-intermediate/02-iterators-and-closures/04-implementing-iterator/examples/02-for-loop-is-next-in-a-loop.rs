//! A `for` loop over any iterator is exactly this: a `loop` that calls
//! `.next()` and stops at the first `None`. Nothing more is happening — the
//! compiler writes this same `loop` for you every time you write `for`.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 02-for-loop-is-next-in-a-loop

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
    println!("written by hand, calling .next() ourselves:");
    let mut manual = Fibonacci::new().take(6);
    loop {
        match manual.next() {
            Some(value) => println!("  {value}"),
            None => break,
        }
    }

    println!("the same six values, written as a for loop:");
    for value in Fibonacci::new().take(6) {
        println!("  {value}");
    }
}
