//! DELIBERATELY BROKEN — expected: E0369
//!
//! `largest` with the `PartialOrd` bound removed. The compiler will not
//! assume an unbound `T` supports `>` — it has no idea what `T` will end up
//! being, so it refuses to guess.
//!
//!     cargo run -p p2-03-02-generic-functions-and-structs --example 04-missing-bound --features broken

fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let episode_counts = [12, 24, 13, 64, 51];
    println!("most episodes: {}", largest(&episode_counts));
}
