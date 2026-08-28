//! The fix for the E0308 in 10-branch-mismatch-broken.rs: when the real
//! return type depends on a runtime branch, `impl Trait` cannot express it
//! — swap it for `Box<dyn Fn(i32) -> i32>`, a trait object, and both
//! branches are welcome.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 05-branch-fixed-with-box-dyn-fn

fn make_adjuster(bonus: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
    if bonus {
        Box::new(move |x| x + n)
    } else {
        Box::new(move |x| x * n)
    }
}

fn main() {
    let add_five = make_adjuster(true, 5);
    let times_five = make_adjuster(false, 5);
    println!("{}", add_five(1));
    println!("{}", times_five(1));
}
