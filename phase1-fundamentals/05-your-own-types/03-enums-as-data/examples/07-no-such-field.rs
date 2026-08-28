//! DELIBERATELY BROKEN — expected: E0609
//!
//!     cargo run -p p1-05-03-enums-as-data --example 07-no-such-field --features broken
//!
//! Getting the data back out of a variant is what 1.5.4 is for.

#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
}

fn main() {
    println!("{:?}", Entry::Planned);
    println!("{:?}", Entry::Watching(7));

    let entry = Entry::Rated { score: 9 };
    println!("score: {}", entry.score);
}
