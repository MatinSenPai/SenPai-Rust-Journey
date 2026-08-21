//! Reference solution for 1.1.3 — Compound types and destructuring.

/// `total_seconds` split into whole hours, whole minutes and the seconds left.
pub fn split_duration(total_seconds: u32) -> (u32, u32, u32) {
    let hours = total_seconds / 3_600;
    let minutes = total_seconds % 3_600 / 60;
    let seconds = total_seconds % 60;
    (hours, minutes, seconds)
}

/// The inverse of [`split_duration`].
pub fn seconds_from(parts: (u32, u32, u32)) -> u32 {
    let (hours, minutes, seconds) = parts;
    hours * 3_600 + minutes * 60 + seconds
}

/// The temperature out of a `(timestamp, celsius, humidity)` sample.
pub fn celsius_of(sample: (u32, f64, f64)) -> f64 {
    let (_, celsius, _) = sample;
    celsius
}

/// The first and last of five readings.
pub fn endpoints(readings: [i32; 5]) -> (i32, i32) {
    (readings[0], readings[readings.len() - 1])
}

/// The point halfway between `a` and `b`.
pub fn midpoint(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let ((ax, ay), (bx, by)) = (a, b);
    ((ax + bx) / 2.0, (ay + by) / 2.0)
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
        assert_eq!(midpoint((0.0, 0.0), (4.0, 10.0)), (2.0, 5.0));
        assert_eq!(midpoint((-3.0, 1.0), (1.0, 1.0)), (-1.0, 1.0));
        assert_eq!(midpoint((0.5, 0.25), (1.5, 0.75)), (1.0, 0.5));
    }
}
