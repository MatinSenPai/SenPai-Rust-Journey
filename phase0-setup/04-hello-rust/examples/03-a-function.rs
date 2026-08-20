//! A function with parameters and a return value, called from `main`.
//!
//!     cargo run -p p0-04-hello-rust --example 03-a-function

fn main() {
    println!("{}", banner("Frieren", 4));
}

/// A title followed by `stars` star characters.
fn banner(title: &str, stars: usize) -> String {
    format!("{title} {}", "*".repeat(stars))
}
