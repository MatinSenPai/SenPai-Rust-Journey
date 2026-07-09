//! Cargo basics lesson: a tiny library function, exercised by `main.rs` and
//! by the tests below. Don't worry about fully understanding `&str` vs
//! `String` yet — Phase 1 explains that from scratch. For now, just know:
//! `&str` here means "borrow this text, you don't own it."

/// Builds a greeting for `name`, repeated `times` times, one greeting per
/// line (lines joined with `\n`, no trailing newline).
///
/// # Examples
///
/// ```
/// # use p0_03_cargo_basics::format_greeting;
/// assert_eq!(format_greeting("Matin", 1), "Hello, Matin!");
/// assert_eq!(format_greeting("Matin", 2), "Hello, Matin!\nHello, Matin!");
/// ```
pub fn format_greeting(name: &str, times: u32) -> String {
    todo!("build `times` copies of \"Hello, {{name}}!\", joined by newlines")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_once() {
        assert_eq!(format_greeting("Matin", 1), "Hello, Matin!");
    }

    #[test]
    fn greets_multiple_times() {
        assert_eq!(
            format_greeting("Matin", 3),
            "Hello, Matin!\nHello, Matin!\nHello, Matin!"
        );
    }

    #[test]
    fn zero_times_is_empty_string() {
        assert_eq!(format_greeting("Matin", 0), "");
    }
}
