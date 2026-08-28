//! DELIBERATELY BROKEN — expected: E0308
//!
//! `mut` lets you change the value, never the type. Shadowing is the tool
//! for that, and this file shows why they aren't interchangeable.
//!
//!     cargo run -p p1-01-01-variables-mutability-shadowing --example 04-mut-keeps-its-type --features broken

fn main() {
    let mut total = 100;
    total = total.to_string();
    println!("total: {total}");
}
