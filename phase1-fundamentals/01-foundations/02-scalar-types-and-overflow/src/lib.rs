//! Exercises for 1.1.2 — Scalar types and overflow.
//!
//! Every function here is about a decision the type system forces you to make
//! explicitly: what should happen when a number does not fit, and how to
//! compare two values that are only approximately equal.

/// `a` and `b` added, stopping at `u8::MAX` instead of overflowing.
///
/// # Examples
///
/// `clamped_add(200, 50)` returns `250`.
/// `clamped_add(200, 100)` returns `255` — it stops at the top rather than wrapping.
/// `clamped_add(0, 0)` returns `0`.
pub fn clamped_add(a: u8, b: u8) -> u8 {
    todo!("add them in the way that clamps at the maximum instead of overflowing")
}

/// `a` and `b` added, wrapping round past `u8::MAX` on purpose.
///
/// # Examples
///
/// `wrapped_add(200, 50)` returns `250`.
/// `wrapped_add(250, 10)` returns `4` — 255 is followed by 0.
/// `wrapped_add(255, 1)` returns `0`.
pub fn wrapped_add(a: u8, b: u8) -> u8 {
    todo!("add them in the way that deliberately wraps round")
}

/// Whether `a` and `b` are close enough to treat as equal.
///
/// "Close enough" means their difference is smaller than `TOLERANCE`.
///
/// # Examples
///
/// `is_close(0.1 + 0.2, 0.3)` returns `true`, even though `==` says false.
/// `is_close(1.0, 1.5)` returns `false`.
/// `is_close(-2.0, -2.0)` returns `true`.
pub fn is_close(a: f64, b: f64) -> bool {
    todo!("compare the size of the difference against TOLERANCE, ignoring which is larger")
}

/// `celsius` converted to Fahrenheit.
///
/// The formula is `celsius * 9 / 5 + 32`.
///
/// # Examples
///
/// `to_fahrenheit(0.0)` returns `32.0`.
/// `to_fahrenheit(100.0)` returns `212.0`.
/// `to_fahrenheit(-40.0)` returns `-40.0`.
pub fn to_fahrenheit(celsius: f64) -> f64 {
    todo!("apply the formula and return the result")
}

/// How far apart two floating-point numbers may be and still count as equal.
pub const TOLERANCE: f64 = 1e-9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_at_the_top() {
        assert_eq!(clamped_add(200, 50), 250);
        assert_eq!(clamped_add(200, 100), 255);
        assert_eq!(clamped_add(255, 255), 255);
        assert_eq!(clamped_add(0, 0), 0);
    }

    #[test]
    fn wraps_past_the_top() {
        assert_eq!(wrapped_add(200, 50), 250);
        assert_eq!(wrapped_add(250, 10), 4);
        assert_eq!(wrapped_add(255, 1), 0);
    }

    #[test]
    fn compares_floats_with_a_tolerance() {
        assert!(is_close(0.1 + 0.2, 0.3));
        assert!(is_close(-2.0, -2.0));
        assert!(!is_close(1.0, 1.5));
        // Order must not matter.
        assert!(is_close(0.3, 0.1 + 0.2));
    }

    #[test]
    fn converts_temperatures() {
        assert!(is_close(to_fahrenheit(0.0), 32.0));
        assert!(is_close(to_fahrenheit(100.0), 212.0));
        assert!(is_close(to_fahrenheit(-40.0), -40.0));
    }
}
