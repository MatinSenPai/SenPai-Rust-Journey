pub fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

pub fn append_exclamation(s: &mut String) {
    s.push('!');
}

pub fn swap_and_report(a: &mut i32, b: &mut i32) -> String {
    let (orig_a, orig_b) = (*a, *b);
    std::mem::swap(a, b);
    format!("swapped {orig_a} and {orig_b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_vowels() {
        let text = "Hello Rust";
        assert_eq!(count_vowels(text), 3);
        assert_eq!(text.len(), 10);
    }

    #[test]
    fn appends_in_place() {
        let mut s = String::from("hello");
        append_exclamation(&mut s);
        assert_eq!(s, "hello!");
    }

    #[test]
    fn swaps_and_reports() {
        let mut a = 1;
        let mut b = 2;
        let report = swap_and_report(&mut a, &mut b);
        assert_eq!(a, 2);
        assert_eq!(b, 1);
        assert_eq!(report, "swapped 1 and 2");
    }
}
