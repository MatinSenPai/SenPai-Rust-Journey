//! DELIBERATELY BROKEN — expected: E0277
//!
//! `#[derive(Eq)]` only works when every field is itself `Eq`. `f64` is not
//! — the same `NaN` fact behind the `Vec<f64>`/`BinaryHeap<f64>` errors
//! earlier in Phase 2, now hitting a struct of your own instead of a bare
//! `f64`.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 09-derive-eq-needs-field-eq --features broken

#[derive(Debug, PartialEq, Eq)]
struct Rating {
    value: f64,
}

fn main() {
    println!("{:?}", Rating { value: 9.5 });
}
