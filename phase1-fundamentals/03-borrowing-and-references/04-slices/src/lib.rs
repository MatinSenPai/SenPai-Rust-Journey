/// Returns the first `n` bytes of `s` as a slice (assume `n` always lands
/// on a valid character boundary for the test inputs used here).
pub fn first_n_bytes(s: &str, n: usize) -> &str {
    todo!("&s[..n]")
}

/// Returns every element of `nums` except the first and the last, as a
/// slice (no copying).
pub fn middle(nums: &[i32]) -> &[i32] {
    todo!("&nums[1..nums.len() - 1]")
}

/// Sums every element in a slice of `i32`.
pub fn sum_slice(nums: &[i32]) -> i32 {
    todo!("nums.iter().sum()")
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
        // works on an array, a full Vec borrowed as a slice, or a sub-slice:
        let v = vec![10, 20, 30, 40];
        assert_eq!(sum_slice(&v), 100);
        assert_eq!(sum_slice(&v[1..3]), 50);
    }
}
