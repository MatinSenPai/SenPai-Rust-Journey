//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-02-03-clone-and-copy --example 05-cloning-a-reference --features broken

fn main() {
    let lines = vec![String::from("alpha"), String::from("beta")];

    // `first()` hands back a look at the element, wrapped up in case there
    // is not one. Cloning that gives you a copy of the *look*, not of the
    // String — because a reference is itself `Copy`.
    let first: String = lines.first().clone();

    println!("{first}");
}
