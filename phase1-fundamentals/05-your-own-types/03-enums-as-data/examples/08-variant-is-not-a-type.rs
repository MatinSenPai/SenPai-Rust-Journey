//! DELIBERATELY BROKEN — expected: E0573
//!
//!     cargo run -p p1-05-03-enums-as-data --example 08-variant-is-not-a-type --features broken
//!
//! The enum is the type. A variant is one of the values that type can hold.

#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
}

fn show(entry: Entry::Rated) {
    println!("{entry:?}");
}

fn main() {
    println!("{:?}", Entry::Planned);
    println!("{:?}", Entry::Watching(7));
    show(Entry::Rated { score: 9 });
}
