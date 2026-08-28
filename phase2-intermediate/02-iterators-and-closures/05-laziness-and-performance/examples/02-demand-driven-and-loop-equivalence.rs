//! The core claim of this lesson, made concrete: from an UNBOUNDED source,
//! only as many elements as `.take(3)` actually needs are ever pulled
//! through the whole chain — and a hand-written loop doing the identical
//! job takes the exact same number of steps. Neither one is secretly doing
//! more work than the other.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 02-demand-driven-and-loop-equivalence

fn main() {
    let mut map_calls = 0u32;
    let mut filter_calls = 0u32;
    let chain_result: Vec<i32> = (1..)
        .map(|n| {
            map_calls += 1;
            n * 2
        })
        .filter(|n| {
            filter_calls += 1;
            n % 3 == 0
        })
        .take(3)
        .collect();
    println!("chain result: {chain_result:?}");
    println!("chain: map ran {map_calls} times, filter ran {filter_calls} times");

    println!();

    let mut n = 1i32;
    let mut loop_map_calls = 0u32;
    let mut loop_filter_calls = 0u32;
    let mut loop_result = Vec::new();
    while loop_result.len() < 3 {
        loop_map_calls += 1;
        let doubled = n * 2;
        loop_filter_calls += 1;
        if doubled % 3 == 0 {
            loop_result.push(doubled);
        }
        n += 1;
    }
    println!("loop result:  {loop_result:?}");
    println!("loop:  map ran {loop_map_calls} times, filter ran {loop_filter_calls} times");
}
