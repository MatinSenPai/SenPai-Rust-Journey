//! Run three times: `cargo run --release -p p2-07-05-benchmarking-with-criterion --example 01-naive-timing`
//!
//! Watch "run 1" versus "run 2"/"run 3" — and watch all three numbers move
//! between separate runs of this same program. Neither kind of movement
//! means anything is broken.

use std::time::Instant;

use p2_07_05_benchmarking_with_criterion::contains_linear;

fn main() {
    let haystack: Vec<u32> = (0..1_000).collect();

    let start = Instant::now();
    let found = contains_linear(&haystack, 999_999);
    println!("run 1: found={found}  elapsed={:?}", start.elapsed());

    let start = Instant::now();
    let found = contains_linear(&haystack, 999_999);
    println!("run 2: found={found}  elapsed={:?}", start.elapsed());

    let start = Instant::now();
    let found = contains_linear(&haystack, 999_999);
    println!("run 3: found={found}  elapsed={:?}", start.elapsed());
}
