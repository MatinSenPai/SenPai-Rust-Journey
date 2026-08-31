//! DELIBERATELY BROKEN — expected: E0432
//! `to_json` only exists when this crate is compiled with `json-export` on.
//! This target is gated behind the `broken` feature so the default build
//! stays green; run it with only `broken` on (not `json-export`) to see why:
//!
//!     cargo run -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken

use p2_10_03_cargo_features::{to_json, Report};

fn main() {
    let report = Report {
        label: "latency".to_string(),
        count: 3,
        mean: 30.0,
        min: 10,
        max: 60,
    };
    println!("{}", to_json(&report));
}
