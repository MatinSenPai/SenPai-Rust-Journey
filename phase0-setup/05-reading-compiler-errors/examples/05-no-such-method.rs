//! DELIBERATELY BROKEN — expected: E0599
//! Run it and read the error before you read the lesson's explanation:
//!
//!     cargo run -p p0-05-reading-compiler-errors --example 05-no-such-method

fn main() {
    let title: &str = "Frieren";
    println!("length: {}", title.lenght());
}
