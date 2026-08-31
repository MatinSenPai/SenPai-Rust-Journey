//! DELIBERATELY BROKEN — expected: E0603.
//!
//! An example file is compiled as its own crate, exactly like an
//! integration test — it depends on the library the same way an external
//! user would, so it can only reach `pub` items.
//!
//!     cargo run -p p2-07-02-unit-integration-doc-tests --example 02-private-item-is-unreachable --features broken

use p2_07_02_unit_integration_doc_tests::round_to_one_decimal;

fn main() {
    println!("{}", round_to_one_decimal(1.24));
}
