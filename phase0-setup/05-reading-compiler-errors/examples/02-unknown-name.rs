//! DELIBERATELY BROKEN — expected: E0425
//! Run it and read the error before you read the lesson's explanation:
//!
//!     cargo run -p p0-05-reading-compiler-errors --example 02-unknown-name

fn main() {
    println!("items: {}", total_itens(7, 3));
}

fn total_items(orders: u32, per_order: u32) -> u32 {
    orders * per_order
}
