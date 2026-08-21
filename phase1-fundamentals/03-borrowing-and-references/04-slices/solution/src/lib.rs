pub fn first_n_bytes(s: &str, n: usize) -> &str {
    &s[..n]
}

pub fn middle(nums: &[i32]) -> &[i32] {
    &nums[1..nums.len() - 1]
}

pub fn sum_slice(nums: &[i32]) -> i32 {
    nums.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_first_bytes() {
        assert_eq!(first_n_bytes("hello world", 5), "hello");
        assert_eq!(first_n_bytes("rust", 0), "");
    }

    #[test]
    fn finds_middle() {
        assert_eq!(middle(&[1, 2, 3, 4, 5]), &[2, 3, 4]);
        assert_eq!(middle(&[1, 2]), &[] as &[i32]);
    }

    #[test]
    fn sums_a_slice() {
        assert_eq!(sum_slice(&[1, 2, 3]), 6);
        let v = vec![10, 20, 30, 40];
        assert_eq!(sum_slice(&v), 100);
        assert_eq!(sum_slice(&v[1..3]), 50);
    }
}
