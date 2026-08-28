//! A hand-rolled iterator, driven one `.next()` call at a time. All the
//! state a call needs to produce the next number lives in the struct itself
//! — `current` and `next` carry forward from one call to the next.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 01-fibonacci-manual-next

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
    let mut fib = Fibonacci::new();
    for _ in 0..8 {
        println!("{:?}", fib.next());
    }
}
