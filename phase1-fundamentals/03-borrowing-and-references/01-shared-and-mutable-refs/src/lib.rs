/// Counts the vowels in `s`, using only a shared reference — `s` must
/// remain fully usable by the caller after this call.
pub fn count_vowels(s: &str) -> usize {
    todo!("s.chars().filter(...).count(), checking for a/e/i/o/u (either case)")
}

/// Appends `"!"` to `s` *in place*, through a mutable reference. Returns
/// nothing — the caller's own `String` is what gets modified.
pub fn append_exclamation(s: &mut String) {
    todo!("s.push('!') — a single char, so clippy prefers push() over push_str()")
}

/// Swaps the values `a` and `b` point to (through two separate mutable
/// references — allowed, since they borrow two *different* values, not the
/// same one), and returns a description of what changed.
pub fn swap_and_report(a: &mut i32, b: &mut i32) -> String {
    todo!(
        "swap *a and *b (std::mem::swap, or by hand with a temporary), then format! a description"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_vowels() {
        let text = "Hello Rust";
        assert_eq!(count_vowels(text), 3);
        assert_eq!(text.len(), 10); // still usable — proves we only borrowed it
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
