//! Exercises for 1.1.3 — Compound types and destructuring.
//!
//! Every function here either builds a compound value or takes one apart. None
//! of them needs a loop or a condition; if you reach for one, re-read the
//! signature — the shape of the value is doing the work.

/// `total_seconds` split into whole hours, whole minutes and the seconds left.
///
/// The three parts always add back up to the original: an answer of
/// `(h, m, s)` means `h * 3600 + m * 60 + s == total_seconds`. So `minutes` is
/// never 60 or more, and neither is `seconds`.
///
/// # Examples
///
/// `split_duration(9_045)` returns `(2, 30, 45)`.
/// `split_duration(59)` returns `(0, 0, 59)`.
/// `split_duration(3_600)` returns `(1, 0, 0)`.
pub fn split_duration(total_seconds: u32) -> (u32, u32, u32) {
    todo!("work out each part and return all three as one value")
}

/// The inverse of [`split_duration`]: `(hours, minutes, seconds)` as a single
/// count of seconds.
///
/// # Examples
///
/// `seconds_from((2, 30, 45))` returns `9_045`.
/// `seconds_from((0, 0, 59))` returns `59`.
/// `seconds_from((1, 0, 0))` returns `3_600`.
pub fn seconds_from(parts: (u32, u32, u32)) -> u32 {
    todo!("take the three parts apart and add them back up")
}

/// The temperature out of a sensor sample.
///
/// A sample is `(unix_timestamp, celsius, humidity_percent)`, in that order.
/// Only the temperature is wanted here.
///
/// # Examples
///
/// `celsius_of((1_700_000_000, 21.5, 48.0))` returns `21.5`.
/// `celsius_of((0, -12.25, 90.0))` returns `-12.25`.
pub fn celsius_of(sample: (u32, f64, f64)) -> f64 {
    todo!("return the temperature, naming only the part you need")
}

/// The first and last of five readings, in that order.
///
/// # Examples
///
/// `endpoints([12, 7, 19, 3, 14])` returns `(12, 14)`.
/// `endpoints([5, 5, 5, 5, 5])` returns `(5, 5)`.
/// `endpoints([-1, 0, 0, 0, 9])` returns `(-1, 9)`.
pub fn endpoints(readings: [i32; 5]) -> (i32, i32) {
    todo!("reach for both ends of the array and pair them up")
}

/// The point halfway between `a` and `b`.
///
/// Each point is `(x, y)`. The midpoint's `x` is halfway between the two `x`
/// values, and likewise for `y`.
///
/// # Examples
///
/// `midpoint((0.0, 0.0), (4.0, 10.0))` returns `(2.0, 5.0)`.
/// `midpoint((-3.0, 1.0), (1.0, 1.0))` returns `(-1.0, 1.0)`.
pub fn midpoint(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    todo!("take both points apart, average each coordinate, and pair the results")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_duration_into_three_parts() {
        assert_eq!(split_duration(9_045), (2, 30, 45));
        assert_eq!(split_duration(59), (0, 0, 59));
        assert_eq!(split_duration(3_600), (1, 0, 0));
        assert_eq!(split_duration(0), (0, 0, 0));
        assert_eq!(split_duration(86_399), (23, 59, 59));
    }

    #[test]
    fn rebuilds_a_duration_from_its_parts() {
        assert_eq!(seconds_from((2, 30, 45)), 9_045);
        assert_eq!(seconds_from((0, 0, 59)), 59);
        assert_eq!(seconds_from((1, 0, 0)), 3_600);
        assert_eq!(seconds_from((0, 0, 0)), 0);
    }

    #[test]
    fn splitting_and_rebuilding_gets_back_where_it_started() {
        assert_eq!(seconds_from(split_duration(9_045)), 9_045);
        assert_eq!(seconds_from(split_duration(1)), 1);
        assert_eq!(seconds_from(split_duration(86_399)), 86_399);
    }

    #[test]
    fn picks_the_temperature_out_of_a_sample() {
        assert_eq!(celsius_of((1_700_000_000, 21.5, 48.0)), 21.5);
        assert_eq!(celsius_of((0, -12.25, 90.0)), -12.25);
    }

    #[test]
    fn finds_both_ends_of_the_readings() {
        assert_eq!(endpoints([12, 7, 19, 3, 14]), (12, 14));
        assert_eq!(endpoints([5, 5, 5, 5, 5]), (5, 5));
        assert_eq!(endpoints([-1, 0, 0, 0, 9]), (-1, 9));
    }

    #[test]
    fn finds_the_point_halfway_between() {
        // Every value here is exactly representable in binary, so `assert_eq!`
        // is safe. That is a deliberate choice — see 1.1.2.
        assert_eq!(midpoint((0.0, 0.0), (4.0, 10.0)), (2.0, 5.0));
        assert_eq!(midpoint((-3.0, 1.0), (1.0, 1.0)), (-1.0, 1.0));
        assert_eq!(midpoint((0.5, 0.25), (1.5, 0.75)), (1.0, 0.5));
    }
}
