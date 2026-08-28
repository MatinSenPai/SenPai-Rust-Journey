//! DELIBERATELY BROKEN — expected: a run-time panic, not a compile error.
//!
//!     cargo run -p p1-03-04-slices --example 08-off-the-end --features broken
//!
//! This compiles perfectly. The range is checked when the slice is made,
//! and `end` comes from somewhere the compiler cannot see in advance.

fn main() {
    let readings = vec![10, 20, 30, 40, 50];

    // Fine: entirely inside the Vec.
    let safe = &readings[2..5];
    println!("safe: {safe:?}");

    // `end` could just as easily have come from a file or a request.
    let end = 3 + 5;
    let off = &readings[2..end];
    println!("off:  {off:?}");
}
