//! `.map()` transforms the value inside `Some`. `.and_then()` does the same
//! job for a function that itself returns an `Option` — and does not leave
//! you holding a doubly-wrapped result.
//!
//! Run: `cargo run -p p1-06-02-option-combinators --example 01-map-and-and-then`

fn double_if_positive(n: i32) -> Option<i32> {
    if n > 0 {
        Some(n * 2)
    } else {
        None
    }
}

fn main() {
    let n: Option<i32> = Some(4);

    let mapped: Option<Option<i32>> = n.map(double_if_positive);
    println!("n.map(double_if_positive):      {mapped:?}");

    let chained: Option<i32> = n.and_then(double_if_positive);
    println!("n.and_then(double_if_positive): {chained:?}");

    let negative: Option<i32> = Some(-3);
    println!(
        "Some(-3).and_then(...):         {:?}",
        negative.and_then(double_if_positive)
    );

    let absent: Option<i32> = None;
    println!(
        "None.and_then(...):             {:?}",
        absent.and_then(double_if_positive)
    );

    // A transform that can never itself fail still wants .map(): the closure
    // returns a plain value, not an Option, so there is nothing to flatten.
    let doubled: Option<i32> = n.map(|x| x * 2);
    println!("n.map(|x| x * 2):               {doubled:?}");
}
