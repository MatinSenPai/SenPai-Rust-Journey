//! DELIBERATELY BROKEN -- expected: E0525
//!
//! `apply_twice` needs to call `f` twice, so it requires `Fn`. The closure
//! passed here drops its captured `bonus` -- moving it out of the closure's
//! own captures -- so it only implements `FnOnce`. The hierarchy only goes
//! one direction: every `Fn` is an `FnOnce`, but not every `FnOnce` is a
//! `Fn`, and the compiler will not silently downgrade what `apply_twice`
//! asked for.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 13-fn-bound-rejects-fnonce-only --features broken

fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn main() {
    let bonus = String::from("bonus");
    let consume_and_add = move |x: i32| {
        drop(bonus);
        x + 1
    };
    println!("{}", apply_twice(consume_and_add, 5));
}
