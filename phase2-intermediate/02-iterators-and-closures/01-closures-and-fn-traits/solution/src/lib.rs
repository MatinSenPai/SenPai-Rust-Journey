//! Solution for 2.2.1 — closures, `Fn`/`FnMut`/`FnOnce`, and `move`.

/// Calls `f` on `x`, then calls `f` again on the result of that first call.
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

/// Returns a closure that multiplies its argument by `factor`.
pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |value| value * factor
}

/// Returns how many strings in `items` make `predicate` return `true`.
pub fn count_matching<F: Fn(&str) -> bool>(items: &[String], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

/// Calls `f` exactly `n` times in a row.
pub fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}

/// Calls `f` exactly once and returns whatever it produces.
pub fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
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
