//! Every function here already works and is already tested. Your job is to
//! run `cargo clippy` and clean up each function's style without changing
//! its behavior. Don't remove the doc comments — they explain *what* each
//! function should keep doing after you refactor it.

/// Returns true if `name` is an empty string.
pub fn is_empty_name(name: &str) -> bool {
    name.is_empty()
}

/// Doubles a number.
pub fn double_it(x: i32) -> i32 {
    x * 2
}

/// Returns `flag`, unchanged — written the long way on purpose.
pub fn is_true(flag: bool) -> bool {
    if flag {
        true
    } else {
        false
    }
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// Counts the characters in `s`.
pub fn shout_length(s: &str) -> usize {
    char_count(s)
}

/// Sums every number in `nums`.
/// nums = [1, 2, 3, 4, 5]
pub fn sum_all(nums: &[i32]) -> i32 {
    let mut total = 0;
    for &i in nums {
        total += i;
    }
    total
}

/// Doubles every number in `nums`.
pub fn double_all(nums: &[i32]) -> Vec<i32> {
    nums.iter().copied().map(|x| double_it(x)).collect()
}

/// Returns the value inside `opt`, or `0` if it's `None`.
pub fn get_or_zero(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name() {
        assert!(is_empty_name(&String::new()));
        assert!(!is_empty_name(&String::from("Matin")));
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
