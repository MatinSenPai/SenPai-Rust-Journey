//! DELIBERATELY BROKEN — expected: E0502
//! Run it and read the error; the fixes are in example 03.
//!
//!     cargo run -p p1-03-02-borrow-checker-rules --example 04-push-while-iterating --features broken

fn main() {
    let mut names = vec![String::from("Matin"), String::from("Sara")];

    // `&names` is borrowed for the whole loop, because the loop reads the
    // next element out of it on every turn. `push` needs the Vec to itself.
    for name in &names {
        if name.len() > 4 {
            names.push(String::from("someone"));
        }
    }

    println!("{names:?}");
}
