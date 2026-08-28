//! A plain `fn` item has nothing to capture, so it trivially satisfies
//! `Fn`, `FnMut`, and `FnOnce` all at once -- it can go anywhere a closure
//! is expected.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 08-function-pointers

fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn add_ten(x: i32) -> i32 {
    x + 10
}

fn main() {
    // Pass the named function straight in -- it coerces to the `F: Fn(i32)
    // -> i32` bound with nothing capture-related to infer.
    println!("{}", apply_twice(add_ten, 1));

    // The type of a non-capturing function used as a value: `fn(i32) -> i32`.
    // Note the lowercase `fn` -- a concrete type, not the trait `Fn`.
    let as_pointer: fn(i32) -> i32 = add_ten;
    println!("{}", apply_twice(as_pointer, 1));
}
