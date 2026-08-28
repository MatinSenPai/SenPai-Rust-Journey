//! DELIBERATELY BROKEN — expected: E0061
//! Run it and read the error before you read the lesson's explanation:
//!
//!     cargo run -p p0-05-reading-compiler-errors --example 04-wrong-arity

fn main() {
    println!("items: {}", total_items(7));
}

fn total_items(orders: u32, per_order: u32) -> u32 {
    orders * per_order
}
