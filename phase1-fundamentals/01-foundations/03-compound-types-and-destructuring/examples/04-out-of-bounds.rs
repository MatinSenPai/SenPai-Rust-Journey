//! DELIBERATELY BROKEN — expected: a run-time panic, "index out of bounds".
//!
//! Unlike the others, this one compiles cleanly. It fails when it runs.
//!
//!     cargo run -p p1-01-03-compound-types-and-destructuring --example 04-out-of-bounds --features broken
//!
//! Try it in release as well. Unlike the overflow panic in 1.1.2, this one is
//! still there:
//!
//!     cargo run --release -p p1-01-03-compound-types-and-destructuring --example 04-out-of-bounds --features broken

fn main() {
    let readings = [12, 7, 19, 3, 14];
    println!("length:  {}", readings.len());

    // `black_box` hides the value from the compiler. Without it rustc notices
    // that the index is always 5, and refuses to compile the program at all —
    // which is a different error, and also in the README.
    let wanted = std::hint::black_box(5);

    println!("safely:  {:?}", readings.get(wanted));
    println!("index:   {}", readings[wanted]);
    println!("this line is never reached");
}
