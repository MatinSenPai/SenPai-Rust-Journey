//! Reference solution for 1.6.4 — panic versus `Result`.
//!
//! Four functions. Two take input from outside this program and must never
//! panic on it. Two take input this program itself is responsible for and
//! are allowed — expected — to panic if that responsibility was dropped.

/// Parses `input` as a priority level from 1 to 5 inclusive.
///
/// `input` is text a person typed, so a bad value is ordinary, not a bug.
/// Never panics.
///
/// - If `input` (after trimming whitespace) does not parse as a `u8` at
///   all, return `Err(format!("'{input}' is not a whole number"))` — using
///   the original, untrimmed `input` in the message.
/// - If it parses but is outside `1..=5`, return
///   `Err(format!("priority must be between 1 and 5, got {value}"))`.
/// - Otherwise return `Ok(value)`.
///
/// # Examples
///
/// `parse_priority("3")` returns `Ok(3)`.
/// `parse_priority("9")` returns `Err("priority must be between 1 and 5, got 9".to_string())`.
/// `parse_priority("abc")` returns `Err("'abc' is not a whole number".to_string())`.
pub fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}

/// The word for a priority level that has *already* been validated —
/// typically the `Ok` value `parse_priority` handed back.
///
/// This function trusts its caller completely: `level` is documented to
/// always be `1..=5`. An out-of-range `level` here is a bug in the caller,
/// not bad input, so it does not return `Result` — it panics.
///
/// `1` -> `"low"`, `2` -> `"low"`, `3` -> `"normal"`, `4` -> `"high"`,
/// `5` -> `"high"`. Anything else: the invariant was violated — use
/// `unreachable!()` with a message naming what broke.
///
/// # Examples
///
/// `priority_label(1)` returns `"low"`.
/// `priority_label(3)` returns `"normal"`.
/// `priority_label(5)` returns `"high"`.
pub fn priority_label(level: u8) -> &'static str {
    match level {
        1 | 2 => "low",
        3 => "normal",
        4 | 5 => "high",
        other => unreachable!(
            "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
        ),
    }
}

/// The middle element of `sorted_ascending`.
///
/// The caller guarantees this slice is non-empty and sorted in ascending
/// order — this function builds nothing from outside input, only from
/// values this program already assembled. An empty slice means a bug
/// upstream in this program: `assert!` that invariant with a message
/// naming it, then return the element at index `len() / 2`.
///
/// # Examples
///
/// `checked_midpoint(&[10, 20, 30, 40, 50])` returns `30`.
/// `checked_midpoint(&[1, 2])` returns `2` (index `2 / 2 = 1`).
pub fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}

/// The last digit of the last value in `values`.
///
/// The caller guarantees `values` is non-empty. Use `.expect()` on
/// `values.last()` with a message naming *that assumption* — not a message
/// describing what `None` means in general — then return that value's last
/// digit (`value % 10`).
///
/// # Examples
///
/// `last_digit_of(&[7, 23, 144])` returns `4`.
/// `last_digit_of(&[9])` returns `9`.
pub fn last_digit_of(values: &[u32]) -> u32 {
    let last = values
        .last()
        .expect("last_digit_of: caller must not pass an empty values slice");
    last % 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_priorities() {
        assert_eq!(parse_priority("3"), Ok(3));
        assert_eq!(parse_priority(" 1 "), Ok(1));
        assert_eq!(parse_priority("5"), Ok(5));
    }

    #[test]
    fn rejects_out_of_range_priorities_without_panicking() {
        assert_eq!(
            parse_priority("9"),
            Err("priority must be between 1 and 5, got 9".to_string())
        );
        assert_eq!(
            parse_priority("0"),
            Err("priority must be between 1 and 5, got 0".to_string())
        );
    }

    #[test]
    fn rejects_unparsable_priorities_without_panicking() {
        assert_eq!(
            parse_priority("abc"),
            Err("'abc' is not a whole number".to_string())
        );
        assert_eq!(
            parse_priority("-1"),
            Err("'-1' is not a whole number".to_string())
        );
    }

    #[test]
    fn labels_every_valid_priority() {
        assert_eq!(priority_label(1), "low");
        assert_eq!(priority_label(2), "low");
        assert_eq!(priority_label(3), "normal");
        assert_eq!(priority_label(4), "high");
        assert_eq!(priority_label(5), "high");
    }

    #[test]
    #[should_panic]
    fn panics_when_the_caller_skips_validation() {
        priority_label(9);
    }

    #[test]
    fn finds_the_midpoint() {
        assert_eq!(checked_midpoint(&[10, 20, 30, 40, 50]), 30);
        assert_eq!(checked_midpoint(&[1, 2]), 2);
        assert_eq!(checked_midpoint(&[7]), 7);
    }

    #[test]
    #[should_panic(expected = "must not pass an empty slice")]
    fn panics_on_an_empty_slice() {
        checked_midpoint(&[]);
    }

    #[test]
    fn finds_the_last_digit() {
        assert_eq!(last_digit_of(&[7, 23, 144]), 4);
        assert_eq!(last_digit_of(&[9]), 9);
    }

    #[test]
    #[should_panic]
    fn panics_on_empty_values() {
        last_digit_of(&[]);
    }
}
