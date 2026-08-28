//! DELIBERATELY BROKEN — expected: E0308.
//!
//! `impl Trait` in return position promises exactly one concrete type. Here
//! the two branches return two different closures — same signature,
//! different types — and `impl Fn(i32) -> i32` cannot name "either of
//! these."
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 10-branch-mismatch-broken --features broken

fn make_adjuster(bonus: bool, n: i32) -> impl Fn(i32) -> i32 {
    if bonus {
        move |x| x + n
    } else {
        move |x| x * n
    }
}

fn main() {
    let f = make_adjuster(true, 5);
    println!("{}", f(1));
}
