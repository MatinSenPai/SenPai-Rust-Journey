//! Reference solution for 06 — Tooling.
//!
//! Every function does exactly what it did before; only the style changed.
//! `cargo clippy` reports nothing here.

/// True when `name` has no characters.
pub fn is_empty_name(name: &str) -> bool {
    name.is_empty()
}

/// `x` doubled.
pub fn double_it(x: i32) -> i32 {
    x * 2
}

/// `flag`, unchanged.
pub fn is_true(flag: bool) -> bool {
    flag
}

/// The number of characters in `s`.
pub fn shout_length(s: &str) -> usize {
    s.chars().count()
}

/// Every number in `nums`, added together.
pub fn sum_all(nums: &[i32]) -> i32 {
    nums.iter().sum()
}

/// Every number in `nums`, doubled.
pub fn double_all(nums: &[i32]) -> Vec<i32> {
    nums.iter().copied().map(double_it).collect()
}

/// The value inside `opt`, or `0` when there isn't one.
pub fn get_or_zero(opt: Option<i32>) -> i32 {
    opt.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name() {
        assert!(is_empty_name(""));
        assert!(!is_empty_name("Matin"));
    }

    #[test]
    fn doubling() {
        assert_eq!(double_it(21), 42);
    }

    #[test]
    fn identity_bool() {
        assert!(is_true(true));
        assert!(!is_true(false));
    }

    #[test]
    fn counts_characters() {
        assert_eq!(shout_length("rust"), 4);
    }

    #[test]
    fn summing() {
        assert_eq!(sum_all(&[1, 2, 3]), 6);
    }

    #[test]
    fn doubling_all() {
        assert_eq!(double_all(&[1, 2, 3]), vec![2, 4, 6]);
    }

    #[test]
    fn option_defaulting() {
        assert_eq!(get_or_zero(Some(5)), 5);
        assert_eq!(get_or_zero(None), 0);
    }
}
