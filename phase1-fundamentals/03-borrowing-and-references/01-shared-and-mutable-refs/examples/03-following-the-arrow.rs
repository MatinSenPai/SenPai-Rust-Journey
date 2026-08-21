//! `*`, the dot that dereferences for you, and why copying an arrow is cheap.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 03-following-the-arrow

fn main() {
    // A reference to an `i32`. `answer` is the number; `view` is an arrow.
    let answer = 42;
    let view = &answer;

    // `*` follows the arrow. Without it you have the arrow, not the number.
    println!("through *:     {}", *view + 1);
    println!("the arrow  @:  {view:p}");
    println!("the value  @:  {:p}", &answer);

    // For a method call you almost never write `*`, because the dot does it.
    // These two lines are the same call.
    let text = String::from("hello");
    let look = &text;
    println!();
    println!("look.len():    {}", look.len());
    println!("(*look).len(): {}", (*look).len());

    // `{}` in println! follows arrows too, however many there are.
    let arrow_to_arrow = &look;
    println!("two arrows:    {arrow_to_arrow}");
    println!("still counts:  {}", arrow_to_arrow.len());

    // A shared reference is `Copy` — from 1.2.3. Assigning one duplicates
    // the arrow, not the String at the end of it, so both stay usable.
    let first = &text;
    let second = first;
    println!();
    println!("both arrows:   {first} / {second}");
    println!("one buffer:    {}", first.as_ptr() == second.as_ptr());

    // A `&mut` is the one kind of arrow that is *not* Copy: there may only
    // ever be one of it at a time, which is what 1.3.2 is about.
    println!();
    println!("&T is Copy. &mut T is not.");
}
