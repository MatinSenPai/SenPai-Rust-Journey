//! DELIBERATELY BROKEN — expected: E0004
//! Run `cargo run -p p2-10-01-pattern-matching-depth --example 06-slice-missing-arm-broken --features broken`
//! and read the error.
//!
//! Two arms cover length 0 (`[]`) and length 2+ (`[first, .., last]`), but
//! length 1 has no arm at all — the compiler proves the gap instead of
//! letting it panic at runtime.

fn summarize(samples: &[u64]) -> String {
    match samples {
        [] => "no samples".to_string(),
        [first, .., last] => format!("first {first}ms, last {last}ms"),
    }
}

fn main() {
    println!("{}", summarize(&[5, 80, 9, 12]));
}
