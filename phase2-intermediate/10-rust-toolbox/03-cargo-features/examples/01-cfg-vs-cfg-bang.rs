//! Run this twice and compare:
//!
//!     cargo run -p p2-10-03-cargo-features --example 01-cfg-vs-cfg-bang
//!     cargo run -p p2-10-03-cargo-features --example 01-cfg-vs-cfg-bang --features json-export

#[cfg(feature = "json-export")]
fn describe() -> &'static str {
    "compiled in"
}

#[cfg(not(feature = "json-export"))]
fn describe() -> &'static str {
    "compiled out"
}

fn main() {
    println!("describe(): {}", describe());
    println!(
        "cfg!(feature = \"json-export\"): {}",
        cfg!(feature = "json-export")
    );
}
