pub fn safe_add_u8(a: u8, b: u8) -> Option<u8> {
    a.checked_add(b)
}

pub fn distance_from_origin(point: (f64, f64)) -> f64 {
    (point.0 * point.0 + point.1 * point.1).sqrt()
}

pub fn sum_three(nums: [i32; 3]) -> i32 {
    nums.iter().sum()
}

pub fn utf8_len(c: char) -> usize {
    c.len_utf8()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_add_handles_overflow() {
        assert_eq!(safe_add_u8(10, 20), Some(30));
        assert_eq!(safe_add_u8(255, 1), None);
    }

    #[test]
    fn distance_works() {
        assert_eq!(distance_from_origin((3.0, 4.0)), 5.0);
        assert_eq!(distance_from_origin((0.0, 0.0)), 0.0);
    }

    #[test]
    fn sums_three() {
        assert_eq!(sum_three([1, 2, 3]), 6);
        assert_eq!(sum_three([-1, 0, 1]), 0);
    }

    #[test]
    fn utf8_lengths() {
        assert_eq!(utf8_len('a'), 1);
        assert_eq!(utf8_len('é'), 2);
        assert_eq!(utf8_len('🦀'), 4);
    }
}
