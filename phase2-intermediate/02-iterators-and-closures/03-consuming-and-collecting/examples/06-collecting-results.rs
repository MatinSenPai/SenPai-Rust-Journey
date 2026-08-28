//! `Result<T, E>` implements `FromIterator<Result<T, E>>` for `Result<Vec<T>,
//! E>` — collecting an iterator of `Result`s can hand you back a single
//! `Result` wrapping a `Vec`, instead of a `Vec` full of `Result`s. The
//! `println!` inside the closure proves the short-circuit is real: watch
//! which inputs it never gets to.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 06-collecting-results

fn main() {
    let all_good = ["1", "2", "3"];
    let parsed: Result<Vec<i32>, _> = all_good.iter().map(|s| s.parse::<i32>()).collect();
    println!("all valid:   {parsed:?}");

    let has_a_bad_one = ["1", "x", "3"];
    let parsed: Result<Vec<i32>, _> = has_a_bad_one
        .iter()
        .map(|s| {
            println!("  parsing {s:?}...");
            s.parse::<i32>()
        })
        .collect();
    println!("one invalid: {parsed:?}");
    // Notice `"3"` is never printed as parsed above — `.collect()` stopped
    // pulling from the iterator the moment `"x"` produced an `Err`.
}
