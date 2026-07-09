use p2_06_02_unit_integration_doc_tests_solution::celsius_to_fahrenheit;

#[test]
fn converts_body_temperature() {
    let result = celsius_to_fahrenheit(37.0);
    assert!((result - 98.6).abs() < 0.5);
}

#[test]
fn converts_negative_temperatures() {
    assert_eq!(celsius_to_fahrenheit(-40.0), -40.0);
}
