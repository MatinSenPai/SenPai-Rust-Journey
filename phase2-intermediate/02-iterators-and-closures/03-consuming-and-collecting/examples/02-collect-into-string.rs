//! `String` implements the same trait `Vec` does — `FromIterator` — so
//! `.collect()` can build one directly from an iterator of `char`, or from
//! an iterator of string slices. Either way it is concatenation, with
//! nothing inserted between pieces.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 02-collect-into-string

fn main() {
    let letters = ['R', 'u', 's', 't'];
    let word: String = letters.into_iter().collect();
    println!("chars -> String:   {word}");

    let parts = ["Sen", "pai"];
    let shout: String = parts.into_iter().collect();
    println!("&str parts -> String: {shout}");

    // Adapters run first, same as always — `.filter()` here drops the
    // vowels before `.collect()` ever sees them.
    let consonants_only: String = "Frieren"
        .chars()
        .filter(|c| !"aeiou".contains(*c))
        .collect();
    println!("filtered chars -> String: {consonants_only}");
}
