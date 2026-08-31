//! Macros can't hold a `todo!()` the way a function body can — a macro is
//! just tokens until it's *used*. So every unfinished transcriber below
//! expands to a call to [`unsolved`], which panics at test time. The crate
//! builds; the tests fail until you write the real transcribers.

/// Placeholder called by the unfinished macro arms below.
///
/// It takes the macro's captured expressions as `_args` so the code you
/// pass to a macro is still evaluated (and the compiler still sees the
/// caller's variables as used), then panics — which is what makes
/// `cargo test` fail until you replace the transcribers. Once all three
/// macros are implemented, nothing calls this anymore and you can delete it.
pub fn unsolved<T, Args>(macro_name: &str, _args: Args) -> T {
    panic!("{macro_name} is not implemented yet — rewrite its transcriber in src/lib.rs")
}

/// Builds a `HashMap<String, String>` from `"key" => "value"` pairs:
///
/// ```ignore
/// let m: HashMap<String, String> = string_map! {
///     "hero" => "Frieren",
///     "mage" => "Fern",
/// };
/// ```
///
/// Both arms need real transcribers:
/// - the empty arm (`string_map! {}`) expands to an empty, owned
///   `HashMap<String, String>` — no entries, still fully constructed.
/// - the pairs arm expands to code that builds one owned
///   `HashMap<String, String>`, with one entry per `key => value` pair,
///   keys and values converted to owned `String`s, in the order given —
///   and the whole thing evaluates to that map.
#[macro_export]
macro_rules! string_map {
    () => {
        $crate::unsolved("string_map!", ())
    };
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {
        $crate::unsolved("string_map!", ( $( ($key, $value), )+ ))
    };
}

/// Variadic maximum: `max_of!(3, 9, 7)` — any number of arguments ≥ 1.
///
/// Two arms, both real work:
/// - base case (one argument): the result is that single value.
/// - recursive case (two or more): the result is the larger of the first
///   argument and the maximum of everything after it — call the macro
///   again on the remainder to get that second number.
///
/// (Works for any `Ord` type — which is why the tests use integers, not
/// floats: `f64` is only `PartialOrd`.)
#[macro_export]
macro_rules! max_of {
    ( $only:expr ) => {
        $crate::unsolved("max_of!", $only)
    };
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        $crate::unsolved("max_of!", ($first, $( $rest ),+))
    };
}

/// Wraps an expression with timing: `timed!(work())` evaluates `work()`
/// exactly **once** and expands to a `(result, elapsed)` pair, where
/// `result` is `work()`'s own value and `elapsed` is a `std::time::Duration`
/// — the wall-clock time that single evaluation took.
///
/// The "exactly once" part is the whole point: whatever expression the
/// caller passes must appear in the expansion's token stream exactly one
/// time. Anything that pastes it twice (say, to both log it and return it)
/// would silently run the caller's side effects twice.
#[macro_export]
macro_rules! timed {
    ( $work:expr ) => {
        $crate::unsolved("timed!", $work)
    };
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
