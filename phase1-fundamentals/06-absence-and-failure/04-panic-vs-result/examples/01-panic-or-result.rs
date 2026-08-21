//! Two functions, side by side. Both can fail. Only one of them should ever
//! panic — and the reason is not about the code, it's about *who* can be
//! wrong.

/// Ordinary, expected, external input. A person can type anything.
fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}

/// An internal invariant. This function is never handed user input directly
/// — only a slice some earlier part of *this program* built. If it's empty,
/// that's a bug in this program, not a bad request.
fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    // debug_assert!: costs nothing in release, catches a broken caller here
    // in debug. Compare 1.1.2's overflow check, which made the same trade.
    debug_assert!(
        sorted_ascending.windows(2).all(|pair| pair[0] <= pair[1]),
        "checked_midpoint: caller must pass an ascending slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}

fn main() {
    for text in ["3", "9", "abc"] {
        match parse_priority(text) {
            Ok(level) => println!("parse_priority({text:?}): Ok({level})"),
            Err(message) => println!("parse_priority({text:?}): Err({message:?})"),
        }
    }

    let sorted = [10, 20, 30, 40, 50];
    println!(
        "checked_midpoint(&{sorted:?}): {}",
        checked_midpoint(&sorted)
    );
}
