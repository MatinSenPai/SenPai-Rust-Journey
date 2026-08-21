//! `Option` and `Result` are not built into the language. They are enums,
//! written in the standard library, and you could have written them.
//!
//!     cargo run -p p1-05-03-enums-as-data --example 04-option-is-an-enum

// `{:?}` does read every field below, but dead-code analysis deliberately
// ignores a derived `Debug` — so without this the run is buried in warnings
// about data that is in fact used.
#![allow(dead_code)]

/// Our own "a score, or nothing". One unit variant, one tuple variant.
#[derive(Debug)]
enum MaybeScore {
    Nothing,
    Something(u8),
}

/// Our own "it worked, or here is why not". Two tuple variants carrying
/// different types.
#[derive(Debug)]
enum Outcome {
    Worked(u8),
    Failed(String),
}

fn main() {
    println!("ours:     {:?}", MaybeScore::Something(9));
    println!("ours:     {:?}", MaybeScore::Nothing);
    println!("ours:     {:?}", Outcome::Worked(9));
    println!(
        "ours:     {:?}",
        Outcome::Failed(String::from("no such id"))
    );

    // The standard library's pair. Same two shapes each, different names.
    println!();
    println!("std:      {:?}", Some(9_u8));
    println!("std:      {:?}", None::<u8>);
    println!("std:      {:?}", Ok::<u8, String>(9));
    println!(
        "std:      {:?}",
        Err::<u8, String>(String::from("no such id"))
    );

    // `Some` and `None` are variants of an enum called `Option`, so they can
    // be written out long-hand — exactly like `MaybeScore::Something`. The
    // short names work because the prelude imports them for you.
    println!();
    println!("longhand: {:?}", Option::Some(9_u8));
    println!("longhand: {:?}", Option::<u8>::None);
    println!("longhand: {:?}", Result::<u8, String>::Ok(9));

    // Which is why `.last()` hands back an `Option`: a look at the last item
    // if there is one, and a different shape if there is not.
    let scores: Vec<u8> = vec![7, 9];
    let empty: Vec<u8> = Vec::new();
    println!();
    println!("last:     {:?}", scores.last());
    println!("last:     {:?}", empty.last());
}
