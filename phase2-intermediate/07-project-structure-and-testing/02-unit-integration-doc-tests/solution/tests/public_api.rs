// Everything under tests/ compiles as its own separate crate, depending on
// this library the same way an external user would: only `pub` items are
// reachable.
use p2_07_02_unit_integration_doc_tests_solution::{
    average_celsius, celsius_to_fahrenheit, parse_celsius,
};

#[test]
fn converts_body_temperature() {
    let result = celsius_to_fahrenheit(37.0);
    assert!((result - 98.6).abs() < 0.5);
}

#[test]
fn averages_readings() {
    assert_eq!(average_celsius(&[0.0, 10.0, 20.0]), 10.0);
}

#[test]
fn rejects_unparseable_input() {
    assert!(parse_celsius("not a number").is_err());
}
