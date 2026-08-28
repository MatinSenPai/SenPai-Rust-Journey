//! DELIBERATELY BROKEN — expected: E0382.
//!
//!     cargo run -p p1-02-04-ownership-across-functions --example 04-used-after-passing --features broken

fn main() {
    let name = String::from("Matin");

    println!("length:  {}", consume(name));

    // `consume` took ownership and dropped it at the end of its body. There
    // is nothing here to print.
    println!("name:    {name}");
}

fn consume(text: String) -> usize {
    text.len()
}
