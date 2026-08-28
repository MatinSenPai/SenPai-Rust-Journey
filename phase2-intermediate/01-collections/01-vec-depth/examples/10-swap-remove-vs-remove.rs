//! `.swap_remove(i)` is O(1): it moves the LAST element into the gap and
//! shrinks the `Vec` by one — fast, but it does not preserve order.
//! `.remove(i)` is O(n): it shifts every later element down by one — slower,
//! order intact.
//!
//!     cargo run -p p2-01-01-vec-depth --example 10-swap-remove-vs-remove

fn main() {
    let mut fast = vec!["Frieren", "Bocchi", "Mushoku Tensei", "Made in Abyss"];
    println!("before:              {fast:?}");
    let removed = fast.swap_remove(1);
    println!("swap_remove(1) took: {removed:?}");
    println!("left, order changed: {fast:?}");

    let mut ordered = vec!["Frieren", "Bocchi", "Mushoku Tensei", "Made in Abyss"];
    println!();
    println!("before:              {ordered:?}");
    let removed = ordered.remove(1);
    println!("remove(1) took:      {removed:?}");
    println!("left, order intact:  {ordered:?}");
}
