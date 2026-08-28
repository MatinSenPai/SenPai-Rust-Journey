//! Exercises for 2.2.1 — closures, `Fn`/`FnMut`/`FnOnce`, and `move`.
//!
//! Five functions, one per shape from the lesson: a plain `Fn` parameter, a
//! function returning a closure (`impl Fn`), a second `Fn` parameter over a
//! different signature, an `FnMut` parameter, and an `FnOnce` parameter.

/// Calls `f` on `x`, then calls `f` again on the *result* of that first
/// call, and returns that second result. `f` only ever needs to read
/// whatever it captured (if anything) — it is never asked to change
/// anything between the two calls — so the bound here is the strongest,
/// most restrictive closure trait: `Fn`.
///
/// # Examples
///
/// With a closure that adds 3, `apply_twice(f, 10)` is `f(f(10))` =
/// `f(13)` = `16`.
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    todo!("call f on x, then call f again on whatever that first call returned, and return that second result")
}

/// Returns a closure that multiplies its argument by `factor`.
///
/// `factor` is moved into the returned closure (note the `move` keyword
/// already in the shell below) so the closure can safely outlive this
/// function call — without `move`, the closure would try to borrow
/// `factor`, a local variable that is about to go out of scope the moment
/// `make_multiplier` returns.
///
/// # Examples
///
/// `make_multiplier(3)` returns a closure `g` such that `g(4) == 12` and
/// `g(5) == 15`.
pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    // The closure shell (`move |value| ...`) is already here because an
    // `impl Trait` return type needs at least one concrete closure
    // expression for Rust to infer the hidden type from — a bare `todo!()`
    // as the whole function body is not enough information on its own.
    // Fill in the body.
    move |value| todo!("multiply value by factor and return that product")
}

/// Returns how many strings in `items` make `predicate` return `true`.
/// `items` itself is left unchanged.
///
/// # Examples
///
/// With `items = ["ok", "a", "long enough"]` and a predicate that checks
/// "longer than 2 characters", the count is `1` (only `"long enough"`
/// qualifies).
pub fn count_matching<F: Fn(&str) -> bool>(items: &[String], predicate: F) -> usize {
    todo!("count how many strings in items make predicate return true when called on them")
}

/// Calls `f` exactly `n` times in a row, in order. Calling zero times (`n
/// == 0`) is valid and does nothing.
///
/// `f` must be allowed to change something on every call (a counter, a log
/// it pushes to), which is exactly what `FnMut` promises and `Fn` does not
/// — so the bound here is `FnMut`, the weakest trait that still allows
/// repeated calls.
///
/// # Examples
///
/// Calling `call_n_times` with `n = 3` and a closure that pushes `"tick"`
/// to a `Vec` each time it runs leaves that `Vec` holding three `"tick"`
/// entries.
pub fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    todo!("call f exactly n times in a row; for n == 0, call it zero times")
}

/// Calls `f` exactly once and returns whatever it produces.
///
/// `f` is only ever called a single time here, so this accepts the
/// weakest closure trait of the three: `FnOnce`. That is what makes it
/// legal to pass a closure that *consumes* (moves out) something it
/// captured — a closure like that cannot be `Fn` or `FnMut`, only
/// `FnOnce`, and `run_once` is the kind of function that can still accept
/// it.
///
/// # Examples
///
/// Given a captured `String` `owned` and `run_once(move || owned)`, the
/// return value is that same string, moved out.
pub fn run_once<F: FnOnce() -> String>(f: F) -> String {
    todo!("call f a single time and return whatever it produces")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_twice() {
        assert_eq!(apply_twice(|x| x + 3, 10), 16); // 10 + 3 = 13, then 13 + 3 = 16
    }

    #[test]
    fn applies_twice_with_a_capture() {
        let offset = 5;
        assert_eq!(apply_twice(|x| x + offset, 0), 10);
    }

    #[test]
    fn apply_twice_accepts_a_plain_function_too() {
        fn double(x: i32) -> i32 {
            x * 2
        }
        assert_eq!(apply_twice(double, 3), 12); // 3 * 2 = 6, then 6 * 2 = 12
    }

    #[test]
    fn multiplies_by_the_captured_factor() {
        let triple = make_multiplier(3);
        assert_eq!(triple(4), 12);
        assert_eq!(triple(5), 15);
    }

    #[test]
    fn multiplier_can_be_called_many_times() {
        let double = make_multiplier(2);
        for i in 0..5 {
            assert_eq!(double(i), i * 2);
        }
    }

    #[test]
    fn counts_matching_items() {
        let items = vec!["ok".to_string(), "a".to_string(), "long enough".to_string()];
        let count = count_matching(&items, |s| s.len() > 2);
        assert_eq!(count, 1);
    }

    #[test]
    fn counts_zero_when_nothing_matches() {
        let items = vec!["a".to_string(), "b".to_string()];
        let count = count_matching(&items, |s| s.is_empty());
        assert_eq!(count, 0);
    }

    #[test]
    fn counts_with_a_captured_threshold() {
        let items = vec!["hi".to_string(), "hello".to_string(), "hey".to_string()];
        let min_len = 3;
        let count = count_matching(&items, |s| s.len() >= min_len);
        assert_eq!(count, 2);
    }

    #[test]
    fn calls_n_times_with_fn_mut() {
        let mut count = 0;
        call_n_times(|| count += 1, 5);
        assert_eq!(count, 5);
    }

    #[test]
    fn calls_n_times_pushes_to_vec_in_order() {
        let mut log = Vec::new();
        call_n_times(|| log.push("tick"), 3);
        assert_eq!(log, vec!["tick", "tick", "tick"]);
    }

    #[test]
    fn calls_zero_times_does_nothing() {
        let mut count = 0;
        call_n_times(|| count += 1, 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn run_once_consumes_a_captured_string() {
        let owned = String::from("hello");
        let result = run_once(move || owned);
        assert_eq!(result, "hello");
    }

    #[test]
    fn run_once_also_accepts_a_plain_fn_closure() {
        // Every `Fn` closure is also a valid `FnOnce` closure -- the
        // hierarchy runs in this direction, not the other way around.
        let result = run_once(|| "static".to_string());
        assert_eq!(result, "static");
    }

    #[test]
    fn run_once_returns_a_value_built_from_two_captures() {
        let first = String::from("Fri");
        let second = String::from("eren");
        let result = run_once(move || format!("{first}{second}"));
        assert_eq!(result, "Frieren");
    }
}
