//! DELIBERATELY BROKEN — expected: E0384
//!
//!     cargo run -p p1-01-01-variables-mutability-shadowing --example 03-reassign --features broken

fn main() {
    let orders = 7;
    orders = 8;
    println!("orders: {orders}");
}
