pub fn format_greeting(name: &str, times: u32) -> String {
    (0..times)
        .map(|_| format!("Hello, {name}!"))
        .collect::<Vec<_>>()
        .join("\n")
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
