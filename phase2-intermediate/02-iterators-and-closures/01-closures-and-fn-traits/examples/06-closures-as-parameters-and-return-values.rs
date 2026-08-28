//! Every closure has its own unique, unnameable type (see 01), so a
//! function boundary that deals in "a closure" can never just name a
//! concrete parameter or return type. Two directions, two idioms:
//!
//! - as a parameter: a generic type parameter bounded by the trait needed,
//!   or `impl Trait` in argument position -- pure shorthand for the same
//!   thing.
//! - as a return value: `impl Trait` in return position -- "some concrete
//!   type that implements this, not telling you which one."
//!
//! Module 2.3 covers generics and `impl Trait` properly; this is just the
//! shape, motivated by why closures specifically need it.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 06-closures-as-parameters-and-return-values

fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

// Same signature as `apply_twice`, spelled with argument-position `impl
// Trait` instead of a named generic parameter.
fn apply_twice_v2(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

// Returns "a closure" -- `impl Fn(i32) -> i32` means "some concrete type
// implementing this, I'm not telling you which one." `move` is required:
// without it, the closure would try to borrow `n`, a parameter that is
// about to go out of scope when `make_adder` returns.
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn main() {
    println!("{}", apply_twice(|x| x + 10, 1));
    println!("{}", apply_twice_v2(|x| x + 10, 1));

    let add_five = make_adder(5);
    println!("{}", add_five(1));
    println!("{}", add_five(100));
}
