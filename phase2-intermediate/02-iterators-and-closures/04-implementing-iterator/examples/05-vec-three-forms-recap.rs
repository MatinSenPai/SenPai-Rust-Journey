//! `Vec<T>` already implements `IntoIterator` three separate times — by
//! value, by shared reference, and by mutable reference. This is the
//! convention `WatchList` (examples 04, 08, 09) only bothered to implement
//! once. `for t in &titles` and `for t in titles` are not the same method
//! call under the hood.
//!
//!     cargo run -p p2-02-04-implementing-iterator --example 05-vec-three-forms-recap

fn main() {
    let titles = vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()];

    println!("for t in &titles (shared borrow):");
    for t in &titles {
        println!("  {t}");
    }
    println!("titles still usable, {} entries", titles.len());

    let mut rewatch = titles.clone();
    println!("for t in &mut rewatch (mutable borrow):");
    for t in &mut rewatch {
        t.push_str(" (rewatch)");
    }
    println!("  {rewatch:?}");

    println!("for t in titles (by value, moves it):");
    for t in titles {
        println!("  {t}");
    }
    // `titles` does not exist from here on — examples/08 shows what happens
    // if you try to use it anyway.
}
