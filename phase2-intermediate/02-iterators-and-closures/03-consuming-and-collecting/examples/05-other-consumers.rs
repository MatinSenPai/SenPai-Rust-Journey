//! `.collect()` is the general-purpose consumer, but it is not the only
//! one. When you only need a number, a bool, or a single item back — not a
//! whole new collection — one of these is a more direct match.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 05-other-consumers

fn main() {
    let ratings = [7, 9, 5, 10, 6];

    println!("sum:              {}", ratings.iter().sum::<i32>());
    println!(
        "count:            {}",
        ratings.iter().filter(|&&r| r >= 7).count()
    );
    println!("min:              {:?}", ratings.iter().min());
    println!("max:              {:?}", ratings.iter().max());
    println!("first below 6:    {:?}", ratings.iter().find(|&&r| r < 6));
    println!("all at least 5:   {}", ratings.iter().all(|&r| r >= 5));
    println!("any perfect 10:   {}", ratings.iter().any(|&r| r == 10));
    println!("last:             {:?}", ratings.iter().last());

    // Each of these fully consumes its own iterator — you cannot call two
    // of them on the *same* iterator value; `ratings.iter()` above is
    // re-created fresh for each line because `[T; N]::iter()` is cheap to
    // call again, not because the same iterator was reused.
}
