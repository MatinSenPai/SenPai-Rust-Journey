//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run --example 05-box-str-from-literal-broken --features broken`
//! and read the error.

fn main() {
    let text: Box<str> = Box::new("senpai");
    println!("{text}");
}
