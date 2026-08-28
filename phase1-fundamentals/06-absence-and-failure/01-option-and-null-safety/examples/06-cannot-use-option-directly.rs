//! DELIBERATELY BROKEN — expected: E0308.
//!
//! This is the bug the null reference makes possible in most other
//! languages: a lookup that might not find anything, handed straight to
//! code that assumes it did. Java, C# and Python would compile this and
//! crash the first time `id` doesn't match. Rust won't even build it.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 06-cannot-use-option-directly --features broken

fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Matin"))
    } else {
        None
    }
}

fn greet(name: String) -> String {
    format!("hello, {name}")
}

fn main() {
    let name = find_user(7);
    println!("{}", greet(name));
}
