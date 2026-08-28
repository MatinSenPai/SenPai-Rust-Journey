//! DELIBERATELY BROKEN — expected: a run-time panic, "user 7 should exist in
//! the seed data" — the exact text passed to `.expect()`, and nothing else.
//! Compare that to `07-unwrap-on-none`'s message: this is the whole reason
//! `.expect()` exists.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 08-expect-on-none --features broken

fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Matin"))
    } else {
        None
    }
}

fn main() {
    let name = find_user(7).expect("user 7 should exist in the seed data");
    println!("hello, {name}");
}
