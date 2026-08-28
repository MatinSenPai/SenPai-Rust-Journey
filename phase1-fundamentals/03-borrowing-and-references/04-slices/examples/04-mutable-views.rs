//! `&mut [T]` — a window you can write through.
//!
//!     cargo run -p p1-03-04-slices --example 04-mutable-views

fn main() {
    let mut readings = vec![10, 20, 30, 40, 50];
    println!("before:  {readings:?}");

    // A mutable view of the middle three. Writing through it writes into
    // the Vec, because there is only one buffer and this points at it.
    let window = &mut readings[1..4];
    window[0] = 99;
    window[2] = 77;

    println!("window:  {window:?}");
    println!("after:   {readings:?}");

    // A mutable slice cannot grow or shrink what it looks at — no `push`,
    // no `remove`. Its length is fixed the moment it is made. What it can
    // do is rearrange and overwrite what is already there.
    let all = &mut readings[..];
    all.swap(0, 4);
    println!();
    println!("swapped: {readings:?}");

    let tail = &mut readings[2..];
    tail.sort();
    println!("tail sorted, whole: {readings:?}");

    // The aliasing rule from 1.3.2 has not gone anywhere. A `&mut` view is
    // exclusive for as long as it is alive — and it stops being alive at
    // its last use, which is why these two can sit in one function.
    let front = &mut readings[..2];
    front[0] = 0;
    println!();
    println!("front:   {front:?}");

    let back = &mut readings[3..];
    back[0] = 1;
    println!("back:    {back:?}");
    println!("whole:   {readings:?}");
}
