//! DELIBERATELY BROKEN — expected: E0554
//! Run `cargo run -p p2-07-05-benchmarking-with-criterion --example 03-nightly-bench-attribute --features broken`
//!
//! The built-in `#[bench]` attribute predates `criterion` and looks like the
//! obvious first thing to reach for. It only works on nightly Rust, behind
//! an unstable feature flag — read the error, then see "Errors you will
//! meet" for why that rules it out for this course.

#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn bench_square(b: &mut Bencher) {
    b.iter(|| 7 * 7);
}

fn main() {}
