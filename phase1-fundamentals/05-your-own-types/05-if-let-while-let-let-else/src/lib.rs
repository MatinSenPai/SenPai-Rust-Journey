/// Returns `"rated X/10"` if `maybe_rating` is `Some(x)`, or
/// `"not rated yet"` if it's `None`.
pub fn describe_rating(maybe_rating: Option<u8>) -> String {
    todo!("if let Some(r) = maybe_rating {{ ... }} else {{ ... }}")
}

/// Pops every element off `stack` (mutating it, draining it to empty) and
/// returns their sum.
pub fn sum_stack(mut stack: Vec<i32>) -> i32 {
    todo!("while let Some(top) = stack.pop() {{ total += top }}")
}

/// Pops every element off `nums`, keeping only the positive ones, in the
/// order they were popped (i.e. reverse of `nums`'s original order).
pub fn drain_positive(mut nums: Vec<i32>) -> Vec<i32> {
    todo!("while let Some(n) = nums.pop() {{ if n > 0 {{ positives.push(n) }} }}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_rating() {
        assert_eq!(describe_rating(Some(9)), "rated 9/10");
        assert_eq!(describe_rating(None), "not rated yet");
    }

    #[test]
    fn sums_a_stack() {
        assert_eq!(sum_stack(vec![1, 2, 3]), 6);
        assert_eq!(sum_stack(vec![]), 0);
    }

    #[test]
    fn drains_positive_values() {
        assert_eq!(drain_positive(vec![1, -2, 3, -4, 5]), vec![5, 3, 1]);
        assert_eq!(drain_positive(vec![-1, -2]), Vec::<i32>::new());
    }
}
