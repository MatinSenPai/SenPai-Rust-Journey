//! A program that compiles. Run it first, so you know what "it worked" looks
//! like before you go looking at what "it broke" looks like.
//!
//!     cargo run -p p0-05-reading-compiler-errors --example 01-tour

fn main() {
    let orders = 7;
    let per_order = 3;
    println!("orders: {orders}");
    println!("items:  {}", total_items(orders, per_order));
}

/// Total items across `orders` orders that each contain `per_order` items.
fn total_items(orders: u32, per_order: u32) -> u32 {
    orders * per_order
}
