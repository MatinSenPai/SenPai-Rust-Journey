/// Calls `f` on `x`, then calls `f` again on the *result* of that first
/// call. `f` only needs to read whatever it captured (if anything), so the
/// bound here is the strongest, most restrictive closure trait: `Fn`.
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    todo!("f(f(x))")
}

/// Returns a closure that multiplies its argument by `factor`. `factor` is
/// moved into the closure (note the `move` keyword) so the returned
/// closure can safely outlive this function call -- without `move`, the
/// closure would try to borrow `factor`, a local variable that's about to
/// go out of scope.
pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    // The closure shell (`move |x| ...`) is given because `impl Trait` return
    // types need at least one concrete closure expression for Rust to infer
    // the hidden type from -- a bare `todo!()` as the whole function body
    // isn't enough information. Fill in the arithmetic below.
    move |x| todo!("x * factor")
}

/// Counts how many `items` satisfy `predicate`.
pub fn count_matching<F: Fn(&str) -> bool>(items: &[String], predicate: F) -> usize {
    todo!("items.iter().filter(|s| predicate(s)).count(), or a plain for loop with a counter")
}

/// Calls `f` exactly `n` times. `f` must be `FnMut` (not `Fn`) because a
/// realistic caller wants to mutate something on every call -- e.g.
/// incrementing a counter or pushing to a `Vec` -- and `FnMut` is the
/// weakest closure trait that still allows that.
pub fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    todo!("for _ in 0..n {{ f(); }}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_twice() {
        assert_eq!(apply_twice(|x| x + 3, 10), 16); // 10 + 3 = 13, then 13 + 3 = 16
    }

    #[test]
    fn applies_twice_with_capture() {
        let offset = 5;
        assert_eq!(apply_twice(|x| x + offset, 0), 10);
    }

    #[test]
    fn multiplies_by_captured_factor() {
        let triple = make_multiplier(3);
        assert_eq!(triple(4), 12);
        assert_eq!(triple(5), 15);
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
    fn calls_n_times_with_fn_mut() {
        let mut count = 0;
        call_n_times(|| count += 1, 5);
        assert_eq!(count, 5);
    }

    #[test]
    fn calls_n_times_pushes_to_vec() {
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
}
