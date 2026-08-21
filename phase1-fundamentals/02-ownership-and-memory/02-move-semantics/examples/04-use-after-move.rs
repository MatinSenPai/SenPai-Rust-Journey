//! DELIBERATELY BROKEN — expected: E0382.
//!
//!     cargo run -p p1-02-02-move-semantics --example 04-use-after-move --features broken

fn main() {
    let first = String::from("hello");
    let second = first;

    // `first` handed its buffer to `second`. Using it now would mean two
    // bindings believing they are responsible for one buffer.
    println!("first:  {first}");
    println!("second: {second}");
}
