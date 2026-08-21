//! Every way to write the range, and what a slice will answer.
//!
//!     cargo run -p p1-03-04-slices --example 02-range-forms

fn main() {
    let days = [11, 12, 13, 14, 15, 16, 17];

    // The same ranges you already write in a `for`, used as an index.
    println!("&days[..]     {:?}", &days[..]);
    println!("&days[2..]    {:?}", &days[2..]);
    println!("&days[..3]    {:?}", &days[..3]);
    println!("&days[1..4]   {:?}", &days[1..4]);
    println!("&days[1..=4]  {:?}", &days[1..=4]);
    println!("&days[3..3]   {:?}", &days[3..3]);

    // The start is included and the end is not — `1..4` is three elements,
    // exactly as `for n in 1..4` is three turns.
    let window = &days[1..4];
    println!();
    println!("&days[1..4] has len {}", window.len());

    // What a slice will tell you about itself. `first` and `last` wrap their
    // answer, because a slice is allowed to be empty.
    let week = &days[..];
    println!();
    println!("len:          {}", week.len());
    println!("is_empty:     {}", week.is_empty());
    println!("first:        {:?}", week.first());
    println!("last:         {:?}", week.last());
    println!("contains(14): {}", week.contains(&14));
    println!("contains(99): {}", week.contains(&99));

    // `split_at` cuts one view into two, at the index you give it. Still no
    // allocation: two windows onto the same seven numbers.
    let (front, back) = week.split_at(3);
    println!();
    println!("split_at(3) -> {front:?} and {back:?}");

    // And an empty slice answers all of it without complaining.
    let nothing = &days[4..4];
    println!();
    println!("empty len:    {}", nothing.len());
    println!("empty first:  {:?}", nothing.first());
}
