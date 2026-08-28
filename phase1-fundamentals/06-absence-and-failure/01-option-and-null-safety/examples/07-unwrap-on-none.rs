//! DELIBERATELY BROKEN — expected: a run-time panic, "called
//! `Option::unwrap()` on a `None` value". It compiles cleanly — `.unwrap()`
//! type-checks on any `Option<T>` — and then it dies when you run it.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 07-unwrap-on-none --features broken

fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Matin"))
    } else {
        None
    }
}

fn main() {
    let name = find_user(7).unwrap();
    println!("hello, {name}");
}
