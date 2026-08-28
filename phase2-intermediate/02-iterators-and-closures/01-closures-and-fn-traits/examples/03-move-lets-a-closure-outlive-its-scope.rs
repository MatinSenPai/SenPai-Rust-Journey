//! `move` forces a closure to take ownership of everything it captures,
//! instead of borrowing. You need it exactly when the closure has to
//! outlive the scope it was written in.
//!
//! Without `move` here, `printer` would try to borrow `message`, but
//! `message` is dropped at the end of the inner block, before `printer` is
//! ever called -- see examples/11-missing-move-dangling-borrow.rs
//! (`--features broken`) for the E0597 that produces.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 03-move-lets-a-closure-outlive-its-scope

fn main() {
    let printer;
    {
        let message = String::from("hi from the inner scope");
        printer = move || println!("{message}");
        // `message` itself is unusable from here on -- it was moved into
        // `printer`. That is exactly what makes the next line, outside the
        // block where `message` was declared, still work.
    }
    printer();
}
