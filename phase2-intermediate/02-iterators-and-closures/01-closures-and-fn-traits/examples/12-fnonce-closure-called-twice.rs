//! DELIBERATELY BROKEN -- expected: E0382
//!
//! `consume`'s body moves its captured `name` out (into `owned`), so
//! `consume` only implements `FnOnce`. Calling an `FnOnce` closure moves
//! the closure itself -- so the first `consume()` consumes `consume`, and
//! the second one is a use-after-move on `consume`, not on `name`.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 12-fnonce-closure-called-twice --features broken

fn main() {
    let name = String::from("Rin");
    let consume = move || {
        let owned = name;
        println!("consumed: {owned}");
    };
    consume();
    consume();
}
