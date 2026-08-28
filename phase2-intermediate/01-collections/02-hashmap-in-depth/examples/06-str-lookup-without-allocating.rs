//! `ratings: HashMap<String, V>` owns its keys — but looking one up does not
//! require you to own a `String` too. `.get("literal")` works directly, no
//! `.to_string()` needed just to ask a question.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 06-str-lookup-without-allocating

use std::collections::HashMap;

fn main() {
    let mut ratings: HashMap<String, u8> = HashMap::new();
    ratings.insert(String::from("Frieren"), 10);
    ratings.insert(String::from("Bocchi the Rock"), 9);

    // The map owns `String`s. This lookup key is a plain `&str` literal —
    // no allocation built just to throw away after one comparison. (Example
    // 01 already did this without remarking on it — this is the moment
    // where it is worth stopping to notice.)
    println!("Frieren: {:?}", ratings.get("Frieren"));

    // A `String` you already have works too, borrowed with `&`.
    let name = String::from("Bocchi the Rock");
    println!("{name}: {:?}", ratings.get(&name));
    // `name` is still yours — `.get()` only ever borrows.
    println!("still own it: {name}");
}
