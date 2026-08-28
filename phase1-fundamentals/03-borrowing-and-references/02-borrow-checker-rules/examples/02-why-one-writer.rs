//! Why "one writer" is a rule and not a style guide: `push` is allowed to
//! move the whole buffer, and anything still pointing into the old one would
//! be pointing at memory that has been given back.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 02-why-one-writer

fn main() {
    let mut readings = vec![1, 2, 3];
    println!("start  len/cap {}/{}", readings.len(), readings.capacity());
    println!("       buffer @ {:p}", readings.as_ptr());

    // Room for three, three in it. The next push has to ask the allocator for
    // a bigger block and copy everything across.
    readings.push(4);
    println!("push4  len/cap {}/{}", readings.len(), readings.capacity());
    println!("       buffer @ {:p}", readings.as_ptr());
    println!();
    println!("the address changed: the old block was handed back");
    println!();

    // Now the trap. There is spare room this time, so this push does *not*
    // move anything. Same call, same type, different consequence.
    readings.push(5);
    println!("push5  len/cap {}/{}", readings.len(), readings.capacity());
    println!("       buffer @ {:p}", readings.as_ptr());
    println!();
    println!("same address: this push happened to fit");
    println!();

    // That is the whole argument for a compile-time rule. Whether a push
    // moves the buffer depends on the capacity at run time, so "it worked on
    // my machine" proves nothing. The checker refuses the shape, always.
    let mut roomy: Vec<i32> = Vec::with_capacity(64);
    roomy.push(1);
    println!("roomy  len/cap {}/{}", roomy.len(), roomy.capacity());
    println!("       buffer @ {:p}", roomy.as_ptr());
    roomy.push(2);
    println!("push2  buffer @ {:p}", roomy.as_ptr());
    println!();
    println!("63 more pushes before that one has to move");
}
