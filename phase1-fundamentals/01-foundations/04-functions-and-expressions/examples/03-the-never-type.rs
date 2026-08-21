//! Why `todo!()` compiles in a function that promises to return a number.
//!
//!     cargo run -p p1-01-04-functions-and-expressions --example 03-the-never-type

fn main() {
    // `not_written_yet` claims to return a `u32` and its body is a `todo!()`.
    // That compiles. This program runs and never calls it, so nothing panics.
    println!("this program compiles and runs");
    println!("even though `not_written_yet` has no body worth the name");
    println!("finished:  {}", finished(6));

    // Uncomment to see it panic — the message is the one you wrote.
    // println!("{}", not_written_yet(3));
}

/// The type of `todo!()` is `!`, the never type: the type of an expression
/// that does not produce a value because it does not finish. Since it never
/// produces a value, it can stand in for *any* type — including `u32`.
///
/// That is the whole reason the exercise stubs in this course compile.
#[allow(dead_code, reason = "the point is that it compiles, not that it runs")]
fn not_written_yet(n: u32) -> u32 {
    todo!("multiply n by itself: {n}")
}

/// `panic!`, `unreachable!` and `unimplemented!` are the same trick.
fn finished(n: u32) -> u32 {
    n * n
}
