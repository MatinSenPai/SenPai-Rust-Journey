//! The same job written twice: once the way it comes out of your fingers when
//! you're new to Rust, once the way clippy will nudge you towards. Both
//! produce identical output — run it and see.
//!
//!     cargo run -p p0-06-tooling-clippy-fmt --example 01-before-and-after

fn main() {
    let nums = [3, 1, 4, 1, 5];
    println!("verbose: {}", sum_verbose(&nums));
    println!("idiomatic: {}", sum_idiomatic(&nums));
}

/// What clippy flags: a loop variable used only to index.
#[allow(clippy::needless_range_loop)]
fn sum_verbose(nums: &[i32]) -> i32 {
    let mut total = 0;
    for i in 0..nums.len() {
        total += nums[i];
    }
    total
}

/// What it suggests instead. Same result, and it says what it means.
fn sum_idiomatic(nums: &[i32]) -> i32 {
    nums.iter().sum()
}
