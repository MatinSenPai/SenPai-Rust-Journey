//! Build rung: `Deserialize for Rating`, written by hand with a `Visitor`.

use p3_03_01_serde_depth::Rating;

fn read(json: &str) -> Result<Rating, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

#[test]
fn reads_the_string_form() {
    assert_eq!(read(r#""9/10""#), Ok(Rating(9)));
    assert_eq!(read(r#""10/10""#), Ok(Rating(10)));
    assert_eq!(read(r#""1/10""#), Ok(Rating(1)));
}

#[test]
fn reads_the_integer_form() {
    assert_eq!(read("9"), Ok(Rating(9)));
    assert_eq!(read("10"), Ok(Rating(10)));
}

#[test]
fn out_of_range_integers_use_the_exact_message() {
    assert!(read("0")
        .unwrap_err()
        .contains("rating must be between 1 and 10, got 0"));
    assert!(read("11")
        .unwrap_err()
        .contains("rating must be between 1 and 10, got 11"));
    assert!(read("300")
        .unwrap_err()
        .contains("rating must be between 1 and 10, got 300"));
}

#[test]
fn out_of_range_strings_use_the_exact_message() {
    assert!(read(r#""0/10""#)
        .unwrap_err()
        .contains("rating must be between 1 and 10, got 0"));
    assert!(read(r#""11/10""#)
        .unwrap_err()
        .contains("rating must be between 1 and 10, got 11"));
}

#[test]
fn malformed_strings_are_errors() {
    for bad in [
        r#""nine""#,
        r#""9""#,
        r#""9/11""#,
        r#""/10""#,
        r#""9/10 ""#,
        r#""""#,
    ] {
        assert!(read(bad).is_err(), "{bad} should not parse");
    }
}

#[test]
fn other_json_types_name_what_was_expected() {
    let err = read("true").unwrap_err();
    assert!(
        err.contains(r#"invalid type: boolean `true`, expected a rating like "9/10" or an integer from 1 to 10"#),
        "{err}"
    );
    assert!(read("null").is_err());
    assert!(read("-1").is_err());
    assert!(read("9.5").is_err());
}

#[test]
fn works_inside_other_data() {
    let v: Vec<Rating> = serde_json::from_str(r#"[1,"2/10",10]"#).unwrap();
    assert_eq!(v, vec![Rating(1), Rating(2), Rating(10)]);
}

#[test]
fn write_then_read_is_the_identity() {
    for n in 1..=10 {
        let json = serde_json::to_string(&Rating(n)).unwrap();
        assert_eq!(read(&json), Ok(Rating(n)));
    }
}
