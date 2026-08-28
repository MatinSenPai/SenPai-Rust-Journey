//! `move` only decides *how* a variable is captured (by value instead of by
//! reference). It says nothing about how many times the closure can be
//! called -- that depends only on what the body *does* with the captured
//! value. This closure moves `name` in, but only ever reads it, so it stays
//! callable as many times as you like.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 04-move-does-not-mean-fnonce

fn main() {
    let name = String::from("Rin");
    let greet = move || println!("hello, {name}");

    greet();
    greet();
    greet();
}
