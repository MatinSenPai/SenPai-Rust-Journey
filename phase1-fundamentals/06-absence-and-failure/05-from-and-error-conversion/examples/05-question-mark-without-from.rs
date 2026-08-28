//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-06-05-from-and-error-conversion --example 05-question-mark-without-from --features broken

#[derive(Debug)]
struct ReadingError;

fn parse_id(raw: &str) -> Result<u32, ReadingError> {
    // `raw.parse::<u32>()` fails with `std::num::ParseIntError`, not
    // `ReadingError` — and there is no `From<ParseIntError> for ReadingError`
    // anywhere in this file for `?` to call.
    let id = raw.parse::<u32>()?;
    Ok(id)
}

fn main() {
    println!("{:?}", parse_id("not a number"));
}
