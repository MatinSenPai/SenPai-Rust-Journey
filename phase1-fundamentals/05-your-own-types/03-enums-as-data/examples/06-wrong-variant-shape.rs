//! DELIBERATELY BROKEN — expected: E0533
//!
//!     cargo run -p p1-05-03-enums-as-data --example 06-wrong-variant-shape --features broken
//!
//! Each variant has its own shape, and you build it in that shape.

#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
}

fn main() {
    println!("{:?}", Entry::Planned);
    println!("{:?}", Entry::Watching(7));
    println!("{:?}", Entry::Rated(9));
}
