//! A tuple groups a fixed number of values that do not have to share a type.
//!
//!     cargo run -p p1-01-03-compound-types-and-destructuring --example 01-tuples

fn main() {
    // One value made of three. The type is written the same way it is built.
    let sample: (u32, f64, bool) = (1_700_000_000, 21.5, true);

    // Field access is by position, starting at zero.
    println!("timestamp: {}", sample.0);
    println!("celsius:   {}", sample.1);
    println!("verified:  {}", sample.2);

    // The whole tuple prints with `{:?}` — the debug format. `{}` will not
    // work here, and the error you get for that is in the README.
    println!("whole:     {sample:?}");

    // Position is part of the type. These two are *different types*, even
    // though both are "two numbers".
    let pair: (u32, f64) = (3, 1.5);
    let flipped: (f64, u32) = (1.5, 3);
    println!("pair:      {pair:?}");
    println!("flipped:   {flipped:?}");

    // Tuples nest, and the access chains. The space in `reading.0 .1` is not a
    // typo: written closed up, `.0.1` looks like the float `0.1` to the parser,
    // so rustfmt separates them. `reading.0.1` does compile — the space is just
    // what the formatter settles on, and now you know why when you see it.
    let reading = ((10, 20), true);
    println!("nested .0.1: {}", reading.0 .1);

    // A one-element tuple needs the trailing comma, or it is just a value in
    // brackets.
    let single = (7,);
    println!("single:    {single:?}");

    // And the empty tuple has a name of its own: the unit type, `()`. It is
    // what a function returns when it returns nothing.
    let nothing: () = ();
    println!("unit:      {nothing:?}");
}
