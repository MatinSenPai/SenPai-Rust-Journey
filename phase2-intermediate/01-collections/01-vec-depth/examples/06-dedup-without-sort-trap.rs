//! No panic here, no compile error — just run it and read the output
//! carefully. This is the trap "Errors you will meet" warns you about.
//!
//!     cargo run -p p2-01-01-vec-depth --example 06-dedup-without-sort-trap

fn main() {
    // The same title got logged twice, but a third entry landed between the
    // two — maybe two people were updating the watch list at once.
    let mut logged = vec![
        "Frieren".to_string(),
        "Bocchi".to_string(),
        "Frieren".to_string(),
    ];
    println!("before:              {logged:?}");

    logged.dedup();
    println!("after .dedup():      {logged:?}");
    println!("(still 3 entries — \"Frieren\" was never adjacent to itself)");

    let mut sorted_first = vec![
        "Frieren".to_string(),
        "Bocchi".to_string(),
        "Frieren".to_string(),
    ];
    sorted_first.sort();
    sorted_first.dedup();
    println!();
    println!("sort() then dedup(): {sorted_first:?}");
}
