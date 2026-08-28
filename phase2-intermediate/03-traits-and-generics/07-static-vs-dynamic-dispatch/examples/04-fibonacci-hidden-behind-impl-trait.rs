//! The exact `Fibonacci` type 2.2.4 built, now wrapped by a function that
//! returns `impl Iterator<Item = u64>` instead of naming `Fibonacci`
//! directly. The caller below never sees the real type — and this is still
//! static dispatch: one concrete type, chosen once, known at compile time.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 04-fibonacci-hidden-behind-impl-trait

struct Fibonacci {
    current: u64,
    next: u64,
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

// The return type says "some Iterator of u64", not "Fibonacci". A caller
// reading only this signature has no way to name the real type.
fn fibonacci() -> impl Iterator<Item = u64> {
    Fibonacci {
        current: 0,
        next: 1,
    }
}

fn main() {
    let first_eight: Vec<u64> = fibonacci().take(8).collect();
    println!("{first_eight:?}");
}
