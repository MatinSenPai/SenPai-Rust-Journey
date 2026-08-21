//! DELIBERATELY BROKEN — expected: E0382
//! Run `cargo run --example 06-map-moves-the-option --features broken` and
//! read the error.

fn main() {
    let name: Option<String> = Some("Sam".to_string());
    let length: Option<usize> = name.map(|s| s.len());
    println!("name: {name:?}, length: {length:?}");
}
