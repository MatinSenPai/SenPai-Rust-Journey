//! The same job as `02-error-conversion-by-hand`, same inputs, same outputs
//! — but every call site is a bare `?`. Three `From` impls do the work that
//! `match`/`.map_err()` used to do inline.
//!
//!     cargo run -p p1-06-05-from-and-error-conversion --example 03-error-conversion-with-from

use std::num::ParseFloatError;
use std::num::ParseIntError;

#[derive(Debug)]
enum Unit {
    Celsius,
    Fahrenheit,
}

// A source error does not have to come from the standard library. This one
// is ours, defined two lines below, and it converts exactly the same way.
#[derive(Debug)]
struct UnknownUnit(String);

#[derive(Debug)]
enum ReadingError {
    BadId(ParseIntError),
    BadValue(ParseFloatError),
    BadUnit(UnknownUnit),
}

// One `impl From<Source> for ReadingError` per fallible call `parse_reading`
// makes. `?` finds these on its own — nothing at the call site names them.
impl From<ParseIntError> for ReadingError {
    fn from(err: ParseIntError) -> Self {
        ReadingError::BadId(err)
    }
}

impl From<ParseFloatError> for ReadingError {
    fn from(err: ParseFloatError) -> Self {
        ReadingError::BadValue(err)
    }
}

impl From<UnknownUnit> for ReadingError {
    fn from(err: UnknownUnit) -> Self {
        ReadingError::BadUnit(err)
    }
}

fn parse_unit(raw: &str) -> Result<Unit, UnknownUnit> {
    match raw {
        "C" => Ok(Unit::Celsius),
        "F" => Ok(Unit::Fahrenheit),
        other => Err(UnknownUnit(other.to_string())),
    }
}

fn parse_reading(line: &str) -> Result<(u32, f64, Unit), ReadingError> {
    let mut parts = line.split(',');
    let id_str = parts.next().unwrap_or("");
    let value_str = parts.next().unwrap_or("");
    let unit_str = parts.next().unwrap_or("");

    // No `.map_err()`, no `match` on the `Result` itself. `?` sees an `Err`,
    // calls `From::from` on it, and returns the result — for all three.
    let id = id_str.parse::<u32>()?;
    let value = value_str.parse::<f64>()?;
    let unit = parse_unit(unit_str)?;

    Ok((id, value, unit))
}

fn main() {
    for line in ["12,36.6,C", "x,36.6,C", "12,hot,C", "12,36.6,K"] {
        println!("{line:?} -> {:?}", parse_reading(line));
    }
}
