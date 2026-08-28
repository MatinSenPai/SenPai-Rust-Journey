//! Run: `cargo run --release -p p2-07-05-benchmarking-with-criterion --example 02-why-black-box`
//!
//! Same function, same 100 million calls, same `total` printed at the end —
//! only one loop tells the optimizer the input might change.

use std::hint::black_box;
use std::time::Instant;

fn square(x: u64) -> u64 {
    x * x
}

fn main() {
    const ITERS: u32 = 100_000_000;

    let start = Instant::now();
    let mut total = 0u64;
    for _ in 0..ITERS {
        total += square(black_box(7));
    }
    println!("with black_box:    {:?}  (total={total})", start.elapsed());

    let start = Instant::now();
    let mut total = 0u64;
    for _ in 0..ITERS {
        total += square(7);
    }
    println!("without black_box: {:?}  (total={total})", start.elapsed());
}
