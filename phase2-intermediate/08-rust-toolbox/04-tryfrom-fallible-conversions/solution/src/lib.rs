use std::fmt;

/// The error type shared by every fallible conversion in this lesson.
/// Each variant carries the rejected input back to the caller — they gave
/// us ownership, and handing it back lets them log or reuse it for free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The value was above 100.
    PercentageOutOfRange(u8),
    /// The candidate string failed email validation.
    InvalidEmail(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::PercentageOutOfRange(v) => {
                write!(f, "percentage must be 0-100, got {v}")
            }
            ValidationError::InvalidEmail(s) => write!(f, "not a valid email address: {s:?}"),
        }
    }
}

impl std::error::Error for ValidationError {}

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

    #[test]
    fn validation_errors_render_readable_messages() {
        assert_eq!(
            ValidationError::PercentageOutOfRange(120).to_string(),
            "percentage must be 0-100, got 120"
        );
        assert_eq!(
            ValidationError::InvalidEmail("nope".to_string()).to_string(),
            "not a valid email address: \"nope\""
        );
    }
}
