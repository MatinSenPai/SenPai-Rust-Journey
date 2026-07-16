/// Builds a `HashMap<String, String>` from `"key" => "value"` pairs.
///
/// ```
/// use std::collections::HashMap;
/// use p2_08_02_macro_rules_basics_solution::string_map;
///
/// let m: HashMap<String, String> = string_map! { "hero" => "Frieren" };
/// assert_eq!(m.get("hero").map(String::as_str), Some("Frieren"));
/// ```
#[macro_export]
macro_rules! string_map {
    () => {
        ::std::collections::HashMap::new()
    };
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key.to_string(), $value.to_string()); )+
        map
    }};
}

/// Variadic maximum: `max_of!(3, 9, 7)` — any number of arguments ≥ 1,
/// any `Ord` type. Expands recursively into nested `std::cmp::max` calls.
#[macro_export]
macro_rules! max_of {
    ( $only:expr ) => {
        $only
    };
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        ::std::cmp::max($first, $crate::max_of!( $( $rest ),+ ))
    };
}

/// Wraps an expression with timing: evaluates it exactly once and expands
/// to a `(result, elapsed)` pair, `elapsed` being a `std::time::Duration`.
#[macro_export]
macro_rules! timed {
    ( $work:expr ) => {{
        let start = ::std::time::Instant::now();
        let result = $work;
        (result, start.elapsed())
    }};
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn string_map_builds_a_hashmap_of_owned_strings() {
        let m: HashMap<String, String> = string_map! {
            "hero" => "Frieren",
            "mage" => "Fern",
        };
        assert_eq!(m.len(), 2);
        assert_eq!(m.get("hero").map(String::as_str), Some("Frieren"));
        assert_eq!(m.get("mage").map(String::as_str), Some("Fern"));
    }

    #[test]
    fn string_map_accepts_empty_input_and_trailing_commas() {
        let empty: HashMap<String, String> = string_map! {};
        assert!(empty.is_empty());

        let one: HashMap<String, String> = string_map! { "k" => "v", };
        assert_eq!(one.get("k").map(String::as_str), Some("v"));
    }

    #[test]
    fn max_of_handles_one_and_many_arguments() {
        let single: i32 = max_of!(41);
        assert_eq!(single, 41);

        let many: i32 = max_of!(3, 9, 7, 9, 2);
        assert_eq!(many, 9);
    }

    #[test]
    fn max_of_works_with_expressions_not_just_literals() {
        let a = 10;
        let b = 4;
        let result: i32 = max_of!(a * 2, b + 3, 7);
        assert_eq!(result, 20);
    }

    #[test]
    fn timed_returns_the_expression_value_and_a_duration() {
        let (value, elapsed): (i32, Duration) = timed!(2 + 3);
        assert_eq!(value, 5);
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn timed_evaluates_the_expression_exactly_once() {
        let mut calls = 0;
        let (value, _elapsed): (i32, Duration) = timed!({
            calls += 1;
            calls * 10
        });
        assert_eq!(value, 10);
        assert_eq!(calls, 1);
    }
}
