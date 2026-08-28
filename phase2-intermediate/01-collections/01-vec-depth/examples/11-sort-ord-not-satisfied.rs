//! DELIBERATELY BROKEN — expected: E0277.
//!
//! `f64` implements `PartialOrd` but never `Ord` — `NaN` cannot be ordered
//! against anything, so there is no total order for `.sort()` to rely on.
//! `.sort()` requires `Ord`. Use `.sort_by()` with a real comparator
//! (`f64::total_cmp`, for instance) instead.
//!
//!     cargo run -p p2-01-01-vec-depth --example 11-sort-ord-not-satisfied --features broken

fn main() {
    let mut ratings: Vec<f64> = vec![9.0, 6.5, 9.5, 7.5];
    ratings.sort();
    println!("{ratings:?}");
}
