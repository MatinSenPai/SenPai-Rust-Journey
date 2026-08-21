//! A slice borrows a run of elements. It copies nothing.
//!
//!     cargo run -p p1-03-04-slices --example 01-a-view-not-a-copy

fn main() {
    let readings = vec![10, 20, 30, 40, 50];

    // Three elements out of the middle. Nothing was allocated to make this.
    let middle = &readings[1..4];

    println!("readings: {readings:?}");
    println!("middle:   {middle:?}");

    // The proof: the view's first element is at the *same address* as the
    // Vec's second element. It is the same memory, looked at differently.
    println!();
    println!("readings[1] @: {:p}", &readings[1]);
    println!("middle[0]   @: {:p}", &middle[0]);

    // A slice knows two things and nothing else: where it starts, and how
    // many elements it covers. It does not know it came from a Vec.
    println!();
    println!("readings.len(): {}", readings.len());
    println!("middle.len():   {}", middle.len());

    // Two words wide — a pointer and a length — where a plain `&i32` is one.
    println!();
    println!("size of &[i32]: {}", std::mem::size_of::<&[i32]>());
    println!("size of &i32:   {}", std::mem::size_of::<&i32>());

    // It is a borrow, so everything from the last three lessons still holds:
    // `readings` is readable while the view is alive, and the view stops
    // existing at its last use.
    println!();
    println!("still ours: {readings:?}");
}
