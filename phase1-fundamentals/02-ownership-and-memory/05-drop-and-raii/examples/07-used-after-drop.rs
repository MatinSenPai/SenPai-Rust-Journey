//! DELIBERATELY BROKEN — expected: E0382
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 07-used-after-drop --features broken

fn main() {
    let text = "a heap buffer".to_string();

    // `drop` is an ordinary function taking its argument by value, so this
    // line is a move like any other.
    drop(text);

    println!("{text}");
}
