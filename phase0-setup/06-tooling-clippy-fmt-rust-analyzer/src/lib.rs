//! Every function here already works and is already tested. Nothing is
//! broken — `cargo test` is green before you touch anything.
//!
//! Your job is to make it *idiomatic*. Run `cargo clippy`, read every
//! warning, and clean each function up **without changing what it returns**.
//! The doc comments say what each one must keep doing.

/// True when `name` has no characters.
pub fn is_empty_name(name: &String) -> bool {
    name.len() == 0
}

/// `x` doubled.
pub fn double_it(x: i32) -> i32 {
    return x * 2;
}

/// `flag`, unchanged — written the long way on purpose.
pub fn is_true(flag: bool) -> bool {
    if flag == true {
        true
    } else {
        false
    }
}

/// The number of characters in `s`.
pub fn shout_length(s: &str) -> usize {
    let counted = s.chars().count();
    counted
}

/// Every number in `nums`, added together.
pub fn sum_all(nums: &[i32]) -> i32 {
    let mut total = 0;
    for i in 0..nums.len() {
        total += nums[i];
    }
    total
}

/// Every number in `nums`, doubled.
pub fn double_all(nums: &[i32]) -> Vec<i32> {
    nums.iter().map(|x| double_it(*x)).collect()
}

/// The value inside `opt`, or `0` when there isn't one.
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
