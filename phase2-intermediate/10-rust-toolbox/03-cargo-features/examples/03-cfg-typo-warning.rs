//! Not broken — this compiles and runs fine, it just warns. `made-up-feature`
//! is never declared in this crate's `Cargo.toml`, and the `unexpected_cfgs`
//! lint catches that at compile time rather than letting a typo silently
//! evaluate to `false` forever.
//!
//!     cargo run -p p2-10-03-cargo-features --example 03-cfg-typo-warning

fn main() {
    println!(
        "cfg!(feature = \"made-up-feature\"): {}",
        cfg!(feature = "made-up-feature")
    );
}
