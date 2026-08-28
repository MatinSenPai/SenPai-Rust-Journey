//! Run: cargo run -p p2-03-02-generic-functions-and-structs --example 01-largest

/// Returns a reference to the largest element in `list`.
///
/// Panics if `list` is empty.
fn largest<T: PartialOrd>(list: &[T]) -> &T {
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

    let titles = ["Frieren", "Bocchi the Rock!", "Made in Abyss"];
    println!("alphabetically last: {}", largest(&titles));
}
