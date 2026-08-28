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

/// A percentage proven to be in `0..=100`.
///
/// The inner field is private on purpose: the *only* way to construct a
/// `Percentage` is via `TryFrom`, so holding one is proof the check
/// happened. That guarantee is the whole point of the newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Percentage(u8);

impl Percentage {
    /// Read-only access to the validated value.
    pub fn value(self) -> u8 {
        self.0
    }
}

/// Converts a validated percentage into a fraction: `raw as f64 / 100.0`,
/// where `raw` is the percentage's inner `u8` (`0..=100`). Always
/// succeeds — nothing left to check, since every `Percentage` that exists
/// already passed `TryFrom` on the way in.
impl From<Percentage> for f64 {
    fn from(value: Percentage) -> Self {
        todo!("convert the validated percentage into a fraction between 0.0 and 1.0")
    }
}

impl TryFrom<u8> for Percentage {
    type Error = ValidationError;

    /// Accepts `0..=100`, rejects everything above with
    /// `ValidationError::PercentageOutOfRange` carrying the bad value.
    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        todo!("accept 0 to 100 inclusive as Ok; reject anything above 100 with an Err carrying the rejected value")
    }
}

/// An email address that passed (deliberately minimal) validation:
/// exactly one `@`, with non-empty text on both sides.
///
/// Like `Percentage`, the field is private — `TryFrom<String>` is the
/// only door in.
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

    /// Validation rule: `raw.split_once('@')` must yield a non-empty
    /// local part and a non-empty domain, and the domain must not
    /// contain another `@`.
    ///
    /// Watch the borrow checker here: you can't keep the `&str` halves
    /// from `split_once` alive *and* move `raw` into the result in the
    /// same `match` — decide validity first (a plain `bool` works well),
    /// then build `Ok`/`Err` from the owned `raw` once nothing still
    /// borrows it.
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        todo!(
            "accept a string with exactly one @ and non-empty text on both sides as Ok; reject everything else with an Err carrying the original string"
        )
    }
}

/// Narrows a `u64` to a `u32`. Values that fit come through unchanged;
/// values too large to fit come back as `u32::MAX` instead of truncating
/// bits (as `as` would) or panicking (as a bare `.unwrap()` would).
pub fn saturating_narrow(value: u64) -> u32 {
    todo!("narrow value to a u32; if it fits use it as is, if it's too large use u32::MAX instead")
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
