//! `cargo run -p p2-04-03-deref-asref-borrow --example 03-asref-generic-param`
//!
//! One function body, three calling conventions: a `&str` literal, a
//! borrowed `&String`, and an owned `String` moved in directly. None of them
//! costs `shout` an extra allocation.

fn shout(name: impl AsRef<str>) -> String {
    format!("{}!", name.as_ref().to_uppercase())
}

fn main() {
    println!("{}", shout("frieren"));

    let owned = String::from("bocchi");
    println!("{}", shout(&owned));
    println!("still own it: {owned}");
    println!("{}", shout(owned));
}
