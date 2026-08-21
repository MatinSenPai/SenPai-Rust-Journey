//! `Vec<T>` — the array that grows.
//!
//!     cargo run -p p1-01-06-vec-and-string-basics --example 01-vec-basics

fn main() {
    // Three ways to make one. `Vec::new()` starts empty, so the type has to
    // come from somewhere — either an annotation or a later `push`.
    let mut readings: Vec<i32> = Vec::new();
    let preset = vec![12, 7, 19];
    let zeroed = vec![0_u8; 5];

    println!("empty:     {readings:?}");
    println!("preset:    {preset:?}");
    println!("zeroed:    {zeroed:?}");

    // Growing. `push` puts one value on the end.
    readings.push(12);
    readings.push(7);
    readings.push(19);
    println!("pushed:    {readings:?}");
    println!("length:    {}", readings.len());
    println!("empty?     {}", readings.is_empty());

    // `pop` takes the last one off. It hands back an `Option`, because there
    // might not be a last one.
    println!("popped:    {:?}", readings.pop());
    println!("after pop: {readings:?}");

    // Everything an array could do, a Vec can do.
    println!("first:     {}", readings[0]);
    println!("get(9):    {:?}", readings.get(9));

    // Reading every element without consuming the Vec. The `&` means "let me
    // look, I am not taking it" — 1.3.1 is the whole story.
    let mut total = 0;
    for reading in &readings {
        total += reading;
    }
    println!("total:     {total}");
    println!("still here:{readings:?}");

    // Length is how many are in it. Capacity is how much room was reserved.
    let mut growing: Vec<i32> = Vec::new();
    println!("len / cap: {} / {}", growing.len(), growing.capacity());
    for n in 0..5 {
        growing.push(n);
        println!("  push {n}: {} / {}", growing.len(), growing.capacity());
    }

    // If you know roughly how many are coming, say so and skip the regrowing.
    let sized: Vec<i32> = Vec::with_capacity(100);
    println!("reserved:  {} / {}", sized.len(), sized.capacity());
}
