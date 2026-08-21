//! One function. An array, a Vec, and a piece of either — all accepted.
//!
//!     cargo run -p p1-03-04-slices --example 03-one-parameter-type

fn main() {
    let fixed = [1, 2, 3, 4];
    let grown = vec![10, 20, 30, 40, 50];

    // `total` was written once and never mentioned a length or a Vec.
    println!("array:         {}", total(&fixed));
    println!("vec:           {}", total(&grown));
    println!("part of vec:   {}", total(&grown[1..3]));
    println!("part of array: {}", total(&fixed[..2]));
    println!("nothing:       {}", total(&[]));

    // An array of a different length is a different type — but not a
    // different *slice* type, which is the whole point.
    let longer = [1, 2, 3, 4, 5, 6, 7, 8];
    println!("longer array:  {}", total(&longer));

    // The same machinery you have been using for text all along. `&str` is
    // a string slice: a borrowed view of a run of bytes, two words wide.
    let owned = String::from("hello world");
    let hello = &owned[..5];
    let world = &owned[6..];

    println!();
    println!("owned: {owned}");
    println!("hello: {hello}");
    println!("world: {world}");

    // And `&str` is to `String` exactly what `&[i32]` is to `Vec<i32>`:
    // the parameter type that accepts every one of them.
    println!();
    println!("from a String:  {}", width(&owned));
    println!("from a slice:   {}", width(world));
    println!("from a literal: {}", width("borrowed from the binary"));

    println!();
    println!("size of &str:   {}", std::mem::size_of::<&str>());
    println!("size of &[i32]: {}", std::mem::size_of::<&[i32]>());
}

/// Reads any run of `i32`s, wherever it lives.
fn total(values: &[i32]) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}

/// Reads any run of text, wherever it lives.
fn width(text: &str) -> usize {
    text.len()
}
