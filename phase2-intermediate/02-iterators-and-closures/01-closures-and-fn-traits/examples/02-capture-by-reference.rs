//! The default capture mode is by reference -- shared if the closure body
//! only reads, mutable if the body mutates. The compiler picks whichever the
//! body actually needs; you never say which.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 02-capture-by-reference

fn main() {
    // Only reads `name` -> captured by shared reference (`&String`).
    // `name` is still perfectly usable after the closure is defined and
    // called, exactly like any other shared borrow from Phase 1.
    let name = String::from("Rin");
    let greet = || println!("hello, {name}");
    greet();
    println!("still have `name` here: {name}");

    // Mutates `hits` -> captured by mutable reference (`&mut i32`). Because
    // the *closure binding itself* now holds an exclusive borrow, the
    // binding has to be `mut` too -- the same aliasing rule from 1.3.1,
    // just applied to a variable that holds a closure instead of a `&mut`.
    let mut hits = 0;
    let mut record_hit = || {
        hits += 1;
    };
    record_hit();
    record_hit();
    record_hit();
    println!("hits: {hits}");
}
