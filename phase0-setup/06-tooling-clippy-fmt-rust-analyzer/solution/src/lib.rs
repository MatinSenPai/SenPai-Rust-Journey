pub fn is_empty_name(name: &str) -> bool {
    name.is_empty()
}

pub fn double_it(x: i32) -> i32 {
    x * 2
}

pub fn is_true(flag: bool) -> bool {
    flag
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

pub fn shout_length(s: &str) -> usize {
    char_count(s)
}

pub fn sum_all(nums: &[i32]) -> i32 {
    let mut total = 0;
    for &n in nums {
        total += n;
    }
    total
}

pub fn double_all(nums: &[i32]) -> Vec<i32> {
    nums.iter().copied().map(double_it).collect()
}

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
