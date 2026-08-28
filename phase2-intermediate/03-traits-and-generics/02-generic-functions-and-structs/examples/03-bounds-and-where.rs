//! Run: cargo run -p p2-03-02-generic-functions-and-structs --example 03-bounds-and-where

use std::fmt::Display;

// Same bound, two notations — `largest_inline` and `largest_where` accept
// exactly the same types and do exactly the same thing.
fn largest_inline<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest_where<T>(list: &[T]) -> &T
where
    T: PartialOrd,
{
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// Two bounds on the same parameter: `Display` to format it, `Clone` to hand
// back an owned copy alongside the formatted line.
fn announce<T: Display + Clone>(item: T) -> (String, T) {
    let headline = format!("now airing: {item}");
    (headline, item.clone())
}

fn main() {
    let episodes = [12, 24, 13];
    println!("{}", largest_inline(&episodes));
    println!("{}", largest_where(&episodes));

    let (headline, kept) = announce(String::from("Frieren"));
    println!("{headline}");
    println!("kept a copy: {kept}");
}
