//! `.binary_search()` never checks that the `Vec` is actually sorted — it
//! CANNOT check that, doing so would cost the very O(log n) it exists to
//! save. Call it on unsorted data and you still get an answer back. It just
//! might be the wrong one.
//!
//!     cargo run -p p2-01-01-vec-depth --example 09-binary-search-unsorted-trap

fn main() {
    // NOT sorted by rating — these are in whatever order they were logged.
    let ratings: Vec<f64> = vec![9.0, 6.5, 9.5, 7.5];
    println!("ratings (unsorted): {ratings:?}");

    let target = 7.5;
    match ratings.binary_search_by(|value| value.total_cmp(&target)) {
        Ok(index) => println!("binary_search for {target}: found at index {index}"),
        Err(_) => println!("binary_search for {target}: not found"),
    }

    let mut real_position = None;
    let mut index = 0;
    for value in &ratings {
        if *value == target {
            real_position = Some(index);
        }
        index += 1;
    }
    println!("a real, linear search:  {real_position:?}");
}
