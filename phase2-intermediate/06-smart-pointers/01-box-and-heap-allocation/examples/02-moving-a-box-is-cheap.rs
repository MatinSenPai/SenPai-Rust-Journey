//! Moving a `Box` only ever moves the pointer. The bytes it points at never
//! move — watch the heap address survive the move unchanged.

use std::mem::size_of;

struct BigBuffer {
    data: [u8; 100_000],
}

fn main() {
    let big = Box::new(BigBuffer { data: [0; 100_000] });
    println!("big.data.len() = {}", big.data.len());

    println!("size_of::<BigBuffer>()      = {}", size_of::<BigBuffer>());
    println!(
        "size_of::<Box<BigBuffer>>() = {}",
        size_of::<Box<BigBuffer>>()
    );

    println!("heap address before move: {big:p}");
    let moved = big; // moves the `Box` itself — 8 bytes, not 100,000
    println!("heap address after move:  {moved:p}");
}
