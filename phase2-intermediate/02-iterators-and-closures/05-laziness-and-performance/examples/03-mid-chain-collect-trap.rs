//! The trap: an accidental `.collect()` in the middle of a chain you meant
//! to keep going. It still compiles, and still gives the right answer — it
//! just does a full pass, and a real allocation, that the lazy version
//! never needed.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 03-mid-chain-collect-trap

const SOURCE_LEN: i32 = 100_000;

fn main() {
    // BEFORE: an accidental mid-chain `.collect()`.
    let mut before_map_calls = 0u32;
    let mut before_filter_calls = 0u32;
    let doubled: Vec<i32> = (1..=SOURCE_LEN)
        .map(|n| {
            before_map_calls += 1;
            n * 2
        })
        .collect(); // <- forces every element through `map`, right now
    println!(
        "doubled: {} elements (map ran {before_map_calls} times)",
        doubled.len()
    );

    let first_three: Vec<i32> = doubled
        .into_iter()
        .filter(|n| {
            before_filter_calls += 1;
            n % 3 == 0
        })
        .take(3)
        .collect();
    println!("first_three: {first_three:?} (filter ran {before_filter_calls} times)");

    println!();

    // AFTER: one continuous lazy chain. No intermediate `Vec` ever exists.
    let mut after_map_calls = 0u32;
    let mut after_filter_calls = 0u32;
    let after: Vec<i32> = (1..=SOURCE_LEN)
        .map(|n| {
            after_map_calls += 1;
            n * 2
        })
        .filter(|n| {
            after_filter_calls += 1;
            n % 3 == 0
        })
        .take(3)
        .collect();
    println!(
        "after: {after:?} (map ran {after_map_calls} times, filter ran {after_filter_calls} times)"
    );
}
