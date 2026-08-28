//! DELIBERATELY BROKEN — expected: E0308
//! One character is wrong. Run it and read what the compiler says:
//!
//!     cargo run -p p0-03-hello-rust --example 04-wrong-return --features broken

fn main() {
    println!("{}", banner("Frieren", 4));
}

fn banner(title: &str, stars: usize) -> String {
    format!("{title} {}", "*".repeat(stars));
}
