//! A minimal matcher/transcriber pair, run to see two things: that a
//! captured `expr` keeps its own grouping, and that pasting `$x` twice
//! evaluates the caller's expression twice.

macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

fn main() {
    let a = square!(1 + 2);
    println!("square!(1 + 2) = {a}");

    let mut calls = 0;
    let mut next = || {
        calls += 1;
        calls
    };
    let b = square!(next());
    println!("square!(next()) = {b}, next() ran {calls} times");
}
