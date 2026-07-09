/// `i32` is `Copy`. Take `n`, make a second variable `m` from it, add 10 to
/// `m`, and return both `n` (unchanged) and `m` — proving `n` is still
/// valid after `let m = n`.
pub fn keep_and_modify(n: i32) -> (i32, i32) {
    todo!("let mut m = n; m += 10; return (n, m)")
}

/// `String` is not `Copy`. Use `.clone()` to get an independent duplicate
/// of `tag`, uppercase the *clone*, and return both the original `tag` and
/// the uppercased clone (the original must be untouched, lowercase).
pub fn duplicate_and_uppercase(tag: String) -> (String, String) {
    todo!("let upper = tag.clone().to_uppercase(); return (tag, upper)")
}

/// `Vec<f64>` is not `Copy`, even though `f64` (the type it contains) is —
/// container types are never `Copy` regardless of what they hold. Clone
/// `prices`, double every element in the clone, and return both the
/// original (untouched) and the doubled clone.
pub fn double_prices(prices: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    todo!("clone prices, build a new Vec<f64> with every element * 2.0, return (prices, doubled)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_and_modifies() {
        assert_eq!(keep_and_modify(5), (5, 15));
    }

    #[test]
    fn duplicates_and_uppercases() {
        assert_eq!(
            duplicate_and_uppercase("rust".to_string()),
            ("rust".to_string(), "RUST".to_string())
        );
    }

    #[test]
    fn doubles_prices() {
        let original = vec![1.0, 2.5, 10.0];
        let (kept, doubled) = double_prices(original.clone());
        assert_eq!(kept, original);
        assert_eq!(doubled, vec![2.0, 5.0, 20.0]);
    }
}
