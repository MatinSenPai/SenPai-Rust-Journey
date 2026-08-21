//! Reference solution for 1.6.5 — `From` and error conversion.
//!
//! Two of these write a `From` impl. Two of them use one that already
//! exists. All four lean on the same trick: `?` calls `From::from` on the
//! error it sees, so writing the impl once makes every `?` after it free.

use std::num::ParseIntError;

/// An RGB color, each channel from 0 to 255.
#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Everything that can go wrong turning a string into a [`Color`].
#[derive(Debug, PartialEq)]
pub enum ColorError {
    /// A channel was not a valid `u8` — not a number, empty, or over 255.
    BadChannel(ParseIntError),
    /// The three channels parsed fine but summed to less than 30.
    TooDark,
}

/// Lets `?` turn a channel's [`ParseIntError`] straight into a [`ColorError`].
impl From<ParseIntError> for ColorError {
    fn from(err: ParseIntError) -> Self {
        ColorError::BadChannel(err)
    }
}

/// Parses `s`, formatted as `"r,g,b"`: three comma-separated decimal
/// integers, each 0 to 255 (for example `"255,128,0"`). A field that is
/// missing or not a valid `u8` produces [`ColorError::BadChannel`]; fields
/// beyond the third are ignored.
///
/// If all three channels parse, but `(r as u32) + (g as u32) + (b as u32)` is
/// less than 30, returns [`ColorError::TooDark`] instead of a [`Color`].
///
/// # Examples
///
/// `parse_color("255,128,0")` returns `Ok(Color { r: 255, g: 128, b: 0 })`.
/// `parse_color("10,10,10")` returns `Ok(Color { r: 10, g: 10, b: 10 })` —
/// the sum is exactly 30, which is not *less than* 30.
/// `parse_color("1,1,1")` returns `Err(ColorError::TooDark)`.
/// `parse_color("x,10,10")` returns `Err(ColorError::BadChannel(_))`.
pub fn parse_color(s: &str) -> Result<Color, ColorError> {
    let mut parts = s.split(',');
    let r_str = parts.next().unwrap_or("");
    let g_str = parts.next().unwrap_or("");
    let b_str = parts.next().unwrap_or("");

    let r = r_str.parse::<u8>()?;
    let g = g_str.parse::<u8>()?;
    let b = b_str.parse::<u8>()?;

    if (r as u32) + (g as u32) + (b as u32) < 30 {
        return Err(ColorError::TooDark);
    }

    Ok(Color { r, g, b })
}

/// A temperature in Celsius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Celsius(pub f64);

/// A temperature in Fahrenheit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fahrenheit(pub f64);

/// The standard conversion: `f = c * 9/5 + 32`.
impl From<Celsius> for Fahrenheit {
    fn from(value: Celsius) -> Self {
        Fahrenheit(value.0 * 9.0 / 5.0 + 32.0)
    }
}

/// Every reading in `values`, converted to Fahrenheit, in the same order.
///
/// # Examples
///
/// `all_fahrenheit(vec![Celsius(0.0), Celsius(100.0)])` returns
/// `vec![Fahrenheit(32.0), Fahrenheit(212.0)]`.
/// `all_fahrenheit(vec![])` returns `vec![]`.
pub fn all_fahrenheit(values: Vec<Celsius>) -> Vec<Fahrenheit> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        out.push(value.into());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_bright_color() {
        assert_eq!(
            parse_color("255,128,0"),
            Ok(Color {
                r: 255,
                g: 128,
                b: 0
            })
        );
    }

    #[test]
    fn thirty_exactly_is_not_too_dark() {
        assert_eq!(
            parse_color("10,10,10"),
            Ok(Color {
                r: 10,
                g: 10,
                b: 10
            })
        );
    }

    #[test]
    fn under_thirty_is_too_dark() {
        assert_eq!(parse_color("1,1,1"), Err(ColorError::TooDark));
    }

    #[test]
    fn a_bad_channel_converts_through_from() {
        let expected = "x".parse::<u8>().unwrap_err();
        assert_eq!(
            parse_color("x,10,10"),
            Err(ColorError::BadChannel(expected))
        );
    }

    #[test]
    fn a_missing_channel_is_a_bad_channel_too() {
        let expected = "".parse::<u8>().unwrap_err();
        assert_eq!(parse_color("1,2"), Err(ColorError::BadChannel(expected)));
    }

    #[test]
    fn freezing_and_boiling() {
        assert_eq!(Fahrenheit::from(Celsius(0.0)), Fahrenheit(32.0));
        assert_eq!(Fahrenheit::from(Celsius(100.0)), Fahrenheit(212.0));
    }

    #[test]
    fn minus_forty_is_the_same_in_both() {
        assert_eq!(Fahrenheit::from(Celsius(-40.0)), Fahrenheit(-40.0));
    }

    #[test]
    fn converts_every_reading_in_order() {
        assert_eq!(
            all_fahrenheit(vec![Celsius(0.0), Celsius(100.0)]),
            vec![Fahrenheit(32.0), Fahrenheit(212.0)]
        );
        assert_eq!(all_fahrenheit(vec![]), Vec::<Fahrenheit>::new());
    }

    #[test]
    fn all_fahrenheit_matches_the_same_conversion_used_alone() {
        // If this used its own arithmetic instead of the same `From` impl,
        // a change to the formula above would silently stop applying here.
        let mine = all_fahrenheit(vec![Celsius(37.0)]);
        assert_eq!(mine, vec![Fahrenheit::from(Celsius(37.0))]);
    }
}
