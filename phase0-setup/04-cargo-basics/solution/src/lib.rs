//! Reference solution for 04 — Cargo basics.

/// A greeting for `name`, repeated `times` times, one per line.
pub fn format_greeting(name: &str, times: u32) -> String {
    let one = format!("Hello, {name}!");
    vec![one; times as usize].join("\n")
}

/// One line of encouragement, chosen at random.
pub fn pick_encouragement() -> String {
    let lines = ["Keep going.", "One todo!() at a time.", "You've got this."];
    lines[rand::random::<usize>() % lines.len()].to_string()
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
    fn zero_times_is_an_empty_string() {
        assert_eq!(format_greeting("Matin", 0), "");
    }

    #[test]
    fn encouragement_is_one_of_the_three() {
        let line = pick_encouragement();
        assert!(
            line == "Keep going." || line == "One todo!() at a time." || line == "You've got this.",
            "got an unexpected line: {line}"
        );
    }
}
