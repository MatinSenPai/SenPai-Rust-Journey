//! `&mut T`: an arrow you are allowed to write through.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 02-through-a-mutable-ref

fn main() {
    // The owner has to be `mut` before anybody may borrow it mutably. Change
    // this to `let greeting` and you get E0596 — see 04-owner-not-mut.
    let mut greeting = String::from("hello");

    add_exclamation(&mut greeting);
    println!("after one:     {greeting}");

    add_exclamation(&mut greeting);
    add_exclamation(&mut greeting);
    println!("after three:   {greeting}");

    // The caller's own String changed. `add_exclamation` returns nothing:
    // there is nothing to hand back, because nothing was taken.

    // Through a `&mut i32` you have to write `*` to reach the number. There
    // is no method call for the compiler to hang a dereference on.
    let mut count = 10;
    bump(&mut count);
    bump(&mut count);
    println!();
    println!("count:         {count}");

    // Two `&mut` at the same time is fine when they point at *different*
    // values. The rule in 1.3.2 is about one value, not about counting.
    let mut here = 5;
    let mut there = 0;
    move_one(&mut here, &mut there);
    move_one(&mut here, &mut there);
    println!("moved:         {here} {there}");

    // Iterating a `&mut Vec<i32>` hands you a `&mut i32` each turn, so `*`
    // shows up again inside the loop.
    let mut values = vec![1, 2, 3];
    let buffer = values.as_ptr();
    double_all(&mut values);
    println!("doubled:       {values:?}");

    // And it really was the caller's buffer that changed — same address,
    // no copy out and no copy back.
    println!("same buffer:   {}", buffer == values.as_ptr());
}

/// Changes the caller's String in place. Returns nothing, because there is
/// nothing to give back.
fn add_exclamation(text: &mut String) {
    text.push('!');
}

fn bump(counter: &mut i32) {
    *counter += 1;
}

fn move_one(from: &mut i32, to: &mut i32) {
    *from -= 1;
    *to += 1;
}

fn double_all(values: &mut Vec<i32>) {
    for value in values {
        *value *= 2;
    }
}
