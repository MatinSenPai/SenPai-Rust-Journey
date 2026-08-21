//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! It builds fine. It is the *running* that fails, and only in a debug build.
//! Run it both ways and compare:
//!
//!     cargo run -p p1-01-02-scalar-types-and-overflow --example 04-overflow-panics --features broken
//!     cargo run --release -p p1-01-02-scalar-types-and-overflow --example 04-overflow-panics --features broken

fn main() {
    let mut count: u8 = 250;
    println!("start:      {count}");

    count += 5;
    println!("plus five:  {count}");

    // 255 is the largest value a u8 can hold. One more has nowhere to go.
    count += 1;
    println!("plus one:   {count}");
}
