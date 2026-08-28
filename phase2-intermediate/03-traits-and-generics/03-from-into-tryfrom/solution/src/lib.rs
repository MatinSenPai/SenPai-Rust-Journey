//! Exercises for 2.3.3 — `From`, `Into`, `TryFrom`, `TryInto`.
//!
//! `Percentage` gets both directions: `From<Percentage> for f64` (reading
//! a validated value back out — always succeeds) and `TryFrom<u8> for
//! Percentage` (validating a raw value coming in — can fail). Building
//! both on the same type is the point: it's the *direction* that decides
//! which trait you reach for, not the type.

/// The error every fallible conversion in this lesson can produce. Each
/// variant carries the rejected input back to the caller — they gave us
/// ownership, and handing it back lets them log or reuse it for free.
///
/// This derives only `Debug` for now, not a user-facing message — that's
/// `Display`, and it's next, in 2.3.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The value was above 100.
    PercentageOutOfRange(u8),
    /// The candidate string failed email validation.
    InvalidEmail(String),
}

/// A percentage proven to be in `0..=100`. The inner field is private:
/// `TryFrom` is the only way to construct one, so holding a `Percentage`
/// is proof the check happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Percentage(u8);

impl Percentage {
    /// Read-only access to the validated value.
    pub fn value(self) -> u8 {
        self.0
    }
}

impl From<Percentage> for f64 {
    fn from(value: Percentage) -> Self {
        f64::from(value.0) / 100.0
    }
}

impl TryFrom<u8> for Percentage {
    type Error = ValidationError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if raw <= 100 {
            Ok(Percentage(raw))
        } else {
            Err(ValidationError::PercentageOutOfRange(raw))
        }
    }
}

/// An email address that passed (deliberately minimal) validation:
/// exactly one `@`, with non-empty text on both sides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// The validated address, as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = ValidationError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let valid = matches!(
            raw.split_once('@'),
            Some((local, domain))
                if !local.is_empty() && !domain.is_empty() && !domain.contains('@')
        );

        if valid {
            Ok(EmailAddress(raw))
        } else {
            Err(ValidationError::InvalidEmail(raw))
        }
    }
}

/// Narrows a `u64` to a `u32`, clamping values that don't fit to
/// `u32::MAX` instead of truncating bits (`as`) or panicking (`unwrap`).
pub fn saturating_narrow(value: u64) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_to_f64_divides_by_a_hundred() {
        let zero = Percentage::try_from(0).unwrap();
        let half = Percentage::try_from(50).unwrap();
        let whole = Percentage::try_from(100).unwrap();
        assert_eq!(f64::from(zero), 0.0);
        assert_eq!(f64::from(half), 0.5);
        assert_eq!(f64::from(whole), 1.0);
    }

    #[test]
    fn percentage_to_f64_works_through_into_too() {
        let three_quarters = Percentage::try_from(75).unwrap();
        let fraction: f64 = three_quarters.into();
        assert_eq!(fraction, 0.75);
    }

    #[test]
    fn percentage_accepts_the_full_valid_range() {
        assert_eq!(Percentage::try_from(0).unwrap().value(), 0);
        assert_eq!(Percentage::try_from(55).unwrap().value(), 55);
        assert_eq!(Percentage::try_from(100).unwrap().value(), 100);
    }

    #[test]
    fn percentage_rejects_101_and_up_with_the_offending_value() {
        assert_eq!(
            Percentage::try_from(101),
            Err(ValidationError::PercentageOutOfRange(101))
        );
        assert_eq!(
            Percentage::try_from(255),
            Err(ValidationError::PercentageOutOfRange(255))
        );
    }

    #[test]
    fn implementing_try_from_provides_try_into_for_free() {
        let p: Percentage = 42u8.try_into().unwrap();
        assert_eq!(p.value(), 42);
    }

    #[test]
    fn email_accepts_a_minimal_valid_address() {
        let email = EmailAddress::try_from("fern@example.com".to_string()).unwrap();
        assert_eq!(email.as_str(), "fern@example.com");
    }

    #[test]
    fn email_rejects_malformed_candidates_and_returns_the_input() {
        for bad in ["", "no-at-sign", "@example.com", "fern@", "a@b@c"] {
            assert_eq!(
                EmailAddress::try_from(bad.to_string()),
                Err(ValidationError::InvalidEmail(bad.to_string())),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn saturating_narrow_passes_fitting_values_through() {
        assert_eq!(saturating_narrow(0), 0);
        assert_eq!(saturating_narrow(123_456), 123_456);
        assert_eq!(saturating_narrow(u64::from(u32::MAX)), u32::MAX);
    }

    #[test]
    fn saturating_narrow_clamps_oversized_values() {
        assert_eq!(saturating_narrow(u64::from(u32::MAX) + 1), u32::MAX);
        assert_eq!(saturating_narrow(u64::MAX), u32::MAX);
    }
}
