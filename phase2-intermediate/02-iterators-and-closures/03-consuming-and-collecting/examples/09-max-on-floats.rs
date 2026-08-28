//! DELIBERATELY BROKEN — expected: E0277
//!
//! `.max()` needs every pair of items to have one true answer to "which is
//! bigger" — that guarantee has a name, and `f64` does not make it (`NaN`
//! is comparable to nothing, not even itself).
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 09-max-on-floats --features broken

fn main() {
    let values = vec![1.0, 5.5, 2.3];
    let biggest = values.iter().max();
    println!("{biggest:?}");
}
