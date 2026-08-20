//! DELIBERATELY BROKEN — expected: cannot find macro (this one has no error code)
//! Run it and read what the compiler says:
//!
//!     cargo run -p p0-03-hello-rust --example 05-unknown-macro --features broken

fn main() {
    printn!("Hello, Rust!");
}
