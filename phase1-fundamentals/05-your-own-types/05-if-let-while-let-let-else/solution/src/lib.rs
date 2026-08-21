pub fn describe_rating(maybe_rating: Option<u8>) -> String {
    if let Some(r) = maybe_rating {
        format!("rated {r}/10")
    } else {
        "not rated yet".to_string()
    }
}

pub fn sum_stack(mut stack: Vec<i32>) -> i32 {
    let mut total = 0;
    while let Some(top) = stack.pop() {
        total += top;
    }
    total
}

pub fn drain_positive(mut nums: Vec<i32>) -> Vec<i32> {
    let mut positives = Vec::new();
    while let Some(n) = nums.pop() {
        if n > 0 {
            positives.push(n);
        }
    }
    positives
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
