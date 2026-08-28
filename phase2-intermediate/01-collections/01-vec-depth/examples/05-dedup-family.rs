//! `.dedup()` and its siblings only ever compare NEIGHBORS. A duplicate two
//! or more positions away from its twin survives untouched — that's why you
//! sort first whenever you want every duplicate gone, not just the ones
//! already sitting next to each other.
//!
//!     cargo run -p p2-01-01-vec-depth --example 05-dedup-family

fn main() {
    let mut ratings = vec![7, 7, 9, 9, 9, 5, 7];
    println!("adjacent duplicates only: {ratings:?}");
    ratings.dedup();
    println!("after .dedup():           {ratings:?}");

    let mut titles = vec![
        "bocchi".to_string(),
        "Bocchi".to_string(),
        "frieren".to_string(),
    ];
    println!();
    println!("same show, case differs:  {titles:?}");
    titles.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    println!("after .dedup_by():        {titles:?}");

    let mut logged: Vec<(String, u8)> = vec![
        ("Frieren".to_string(), 9),
        ("Frieren".to_string(), 10), // same show, logged twice — rating typo'd the 2nd time
        ("Mushoku Tensei".to_string(), 8),
    ];
    println!();
    println!("logged twice by title:    {logged:?}");
    logged.dedup_by_key(|entry| entry.0.clone());
    println!("after .dedup_by_key():    {logged:?}");
}
