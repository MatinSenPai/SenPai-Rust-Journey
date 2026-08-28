//! Reference solution for 1.6.2 — `Option` combinators.
//!
//! Six small functions. Each one is naturally a single combinator (or a
//! short chain of them) rather than a `match`.

/// `word`, uppercased with an exclamation mark appended, still wrapped in
/// `Some`. `None` stays `None`.
///
/// # Examples
///
/// `shout(Some("hi"))` returns `Some("HI!")`.
/// `shout(None)` returns `None`.
pub fn shout(word: Option<&str>) -> Option<String> {
    word.map(|w| format!("{}!", w.to_uppercase()))
}

/// `n` divided by two, but only if it divides evenly — an odd `n` has no
/// exact half, so the result is `None`. `None` in, `None` out.
///
/// # Examples
///
/// `safe_half(Some(8))` returns `Some(4)`.
/// `safe_half(Some(7))` returns `None` — 7 does not divide evenly.
/// `safe_half(None)` returns `None`.
pub fn safe_half(n: Option<i32>) -> Option<i32> {
    n.and_then(|x| if x % 2 == 0 { Some(x / 2) } else { None })
}

/// `n` if it is positive, `None` otherwise — including when `n` was already
/// `None`.
///
/// # Examples
///
/// `positive_only(Some(5))` returns `Some(5)`.
/// `positive_only(Some(-2))` returns `None`.
/// `positive_only(Some(0))` returns `None` — zero is not positive.
/// `positive_only(None)` returns `None`.
pub fn positive_only(n: Option<i32>) -> Option<i32> {
    n.filter(|v| *v > 0)
}

/// Whatever is currently in `slot`, removing it. `slot` holds `None`
/// afterward, regardless of what it held before.
///
/// # Examples
///
/// If `slot` holds `Some("draft".to_string())`, this returns
/// `Some("draft".to_string())` and leaves `slot` as `None`.
/// If `slot` already holds `None`, this returns `None` and `slot` stays
/// `None`.
pub fn take_and_reset(slot: &mut Option<String>) -> Option<String> {
    slot.take()
}

/// `x` and `y` combined into one pair, but only if both are present. If
/// either one is missing, the result is `None`.
///
/// # Examples
///
/// `coordinates(Some(3), Some(4))` returns `Some((3, 4))`.
/// `coordinates(Some(3), None)` returns `None`.
/// `coordinates(None, None)` returns `None`.
pub fn coordinates(x: Option<i32>, y: Option<i32>) -> Option<(i32, i32)> {
    x.zip(y)
}

/// `primary` if it holds a value; `backup` otherwise. `backup` is itself an
/// `Option`, so the result can still be `None` if both are.
///
/// # Examples
///
/// `first_available(Some(1), Some(2))` returns `Some(1)`.
/// `first_available(None, Some(2))` returns `Some(2)`.
/// `first_available(None, None)` returns `None`.
pub fn first_available(primary: Option<i32>, backup: Option<i32>) -> Option<i32> {
    primary.or(backup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shouts_the_word() {
        assert_eq!(shout(Some("hi")), Some("HI!".to_string()));
        assert_eq!(shout(Some("")), Some("!".to_string()));
        assert_eq!(shout(None), None);
    }

    #[test]
    fn halves_only_even_numbers() {
        assert_eq!(safe_half(Some(8)), Some(4));
        assert_eq!(safe_half(Some(0)), Some(0));
        assert_eq!(safe_half(Some(7)), None);
        assert_eq!(safe_half(Some(-7)), None);
        assert_eq!(safe_half(None), None);
    }

    #[test]
    fn keeps_only_positive_numbers() {
        assert_eq!(positive_only(Some(5)), Some(5));
        assert_eq!(positive_only(Some(-2)), None);
        assert_eq!(positive_only(Some(0)), None);
        assert_eq!(positive_only(None), None);
    }

    #[test]
    fn take_and_reset_empties_the_slot() {
        let mut slot = Some("draft".to_string());
        assert_eq!(take_and_reset(&mut slot), Some("draft".to_string()));
        assert_eq!(slot, None);

        let mut empty: Option<String> = None;
        assert_eq!(take_and_reset(&mut empty), None);
        assert_eq!(empty, None);
    }

    #[test]
    fn zips_only_when_both_present() {
        assert_eq!(coordinates(Some(3), Some(4)), Some((3, 4)));
        assert_eq!(coordinates(Some(3), None), None);
        assert_eq!(coordinates(None, Some(4)), None);
        assert_eq!(coordinates(None, None), None);
    }

    #[test]
    fn falls_back_only_when_primary_is_absent() {
        assert_eq!(first_available(Some(1), Some(2)), Some(1));
        assert_eq!(first_available(Some(1), None), Some(1));
        assert_eq!(first_available(None, Some(2)), Some(2));
        assert_eq!(first_available(None, None), None);
    }
}
