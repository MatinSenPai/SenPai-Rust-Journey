/// Adds two `u8`s using `checked_add`, which returns `None` instead of
/// panicking on overflow (255 + 1 would panic if you used plain `+`).
pub fn safe_add_u8(a: u8, b: u8) -> Option<u8> {
    todo!("use a.checked_add(b)")
}

/// Given a point as a tuple `(x, y)`, returns the distance from the origin.
pub fn distance_from_origin(point: (f64, f64)) -> f64 {
    todo!("point.0 and point.1, then the usual distance formula: sqrt(x*x + y*y)")
}

/// Returns the sum of a fixed 3-element array of `i32`.
pub fn sum_three(nums: [i32; 3]) -> i32 {
    todo!("add nums[0] + nums[1] + nums[2], or use nums.iter().sum()")
}

/// Returns how many bytes `c` takes when UTF-8 encoded (not the same as
/// "1", even though `char` itself is always 4 bytes in memory).
pub fn utf8_len(c: char) -> usize {
    todo!("char has a method for exactly this — check the std docs for `char`")
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
