//! DELIBERATELY BROKEN -- expected: E0597
//!
//! Without `move`, `printer` only borrows `message`. `message` is dropped
//! at the end of the inner block, but `printer` -- and the borrow it
//! holds -- is still alive outside that block. Fixed version, with `move`:
//! examples/03-move-lets-a-closure-outlive-its-scope.rs
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 11-missing-move-dangling-borrow --features broken

fn main() {
    let printer;
    {
        let message = String::from("hi from the inner scope");
        printer = || println!("{message}");
    }
    printer();
}
