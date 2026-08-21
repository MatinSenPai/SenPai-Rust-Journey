//! One function, three fallible calls, three different error types — folded
//! into one `Result` entirely by hand. No `From` anywhere in this file.
//!
//!     cargo run -p p1-06-05-from-and-error-conversion --example 02-error-conversion-by-hand

use std::num::ParseFloatError;
use std::num::ParseIntError;

#[derive(Debug)]
enum Unit {
    Celsius,
    Fahrenheit,
}

#[derive(Debug)]
enum ReadingError {
    BadId(ParseIntError),
    BadValue(ParseFloatError),
    BadUnit(String),
}

fn parse_unit(raw: &str) -> Result<Unit, String> {
    match raw {
        "C" => Ok(Unit::Celsius),
        "F" => Ok(Unit::Fahrenheit),
        other => Err(other.to_string()),
    }
}

fn parse_reading(line: &str) -> Result<(u32, f64, Unit), ReadingError> {
    let mut parts = line.split(',');
    let id_str = parts.next().unwrap_or("");
    let value_str = parts.next().unwrap_or("");
    let unit_str = parts.next().unwrap_or("");

    // Call one: handled with a full `match`, because that is the only tool
    // for the job so far.
    let id = match id_str.parse::<u32>() {
        Ok(id) => id,
        Err(err) => return Err(ReadingError::BadId(err)),
    };

    // Calls two and three: the same shape, shrunk with `.map_err()` instead
    // of a four-line `match`. Still one line of *conversion* per call.
    let value = value_str.parse::<f64>().map_err(ReadingError::BadValue)?;
    let unit = parse_unit(unit_str).map_err(ReadingError::BadUnit)?;

    Ok((id, value, unit))
}

fn main() {
    for line in ["12,36.6,C", "x,36.6,C", "12,hot,C", "12,36.6,K"] {
        println!("{line:?} -> {:?}", parse_reading(line));
    }
}
