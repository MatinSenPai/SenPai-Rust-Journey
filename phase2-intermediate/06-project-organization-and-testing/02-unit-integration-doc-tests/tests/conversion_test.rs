// Everything in tests/ is compiled as its own separate crate, depending on
// the library the same way an external user would — only `pub` items are
// reachable. `round_to_one_decimal` (private, in src/lib.rs) simply does
// not exist from this file's point of view; there is no path that names
// it, unlike a unit test living inside the crate itself.
use p2_06_02_unit_integration_doc_tests::celsius_to_fahrenheit;

#[test]
fn converts_body_temperature() {
    let result = celsius_to_fahrenheit(37.0);
    assert!((result - 98.6).abs() < 0.5);
}

#[test]
fn converts_negative_temperatures() {
    assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
}
