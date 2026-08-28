//! DELIBERATELY BROKEN — expected: a run-time panic, "readings must not be
//! empty". It compiles cleanly — `average` type-checks no matter what slice
//! you pass it — and then it dies the moment you run it on an empty one.
//! This is the exact panic `average_celsius`'s `should_panic` doc test (in
//! src/lib.rs) asserts happens.
//!
//!     cargo run -p p2-07-02-unit-integration-doc-tests --example 03-average-of-nothing-panics --features broken

fn average(readings: &[f64]) -> f64 {
    assert!(!readings.is_empty(), "readings must not be empty");
    readings.iter().sum::<f64>() / readings.len() as f64
}

fn main() {
    println!("{}", average(&[10.0, 20.0, 30.0]));
    println!("{}", average(&[])); // panics right here
}
