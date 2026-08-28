//! Solution for 2.3.2 — generic functions and structs, bounds, `where`.

/// Returns a reference to the smallest element in `list`.
///
/// Panics if `list` is empty.
pub fn smallest<T: PartialOrd>(list: &[T]) -> &T {
    let mut smallest = &list[0];
    for item in list {
        if item < smallest {
            smallest = item;
        }
    }
    smallest
}

/// Two values of the same type, held side by side.
pub struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    /// Builds a `Pair` holding `first` and `second`.
    pub fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }

    /// A reference to the first value.
    pub fn first(&self) -> &T {
        &self.first
    }

    /// A reference to the second value.
    pub fn second(&self) -> &T {
        &self.second
    }

    /// A reference to whichever value compares larger — `first`, on a tie.
    ///
    /// Only this method needs an ordering, so only this method asks for one.
    pub fn larger(&self) -> &T
    where
        T: PartialOrd,
    {
        if self.second > self.first {
            &self.second
        } else {
            &self.first
        }
    }
}

/// The number of items in `items` for which `predicate` returns `true`.
pub fn matches_count<T, F: Fn(&T) -> bool>(items: &[T], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

/// Returns exactly `format!("label: {item}")`, paired with a clone of `item`.
pub fn label_and_duplicate<T: std::fmt::Display + Clone>(item: T) -> (String, T) {
    let label = format!("label: {item}");
    (label, item.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smallest_finds_min_i32() {
        assert_eq!(smallest(&[3, 7, 2, 9, 4]), &2);
    }

    #[test]
    fn smallest_finds_min_str() {
        assert_eq!(smallest(&["banana", "apple", "cherry"]), &"apple");
    }

    #[test]
    #[should_panic]
    fn smallest_of_empty_slice_panics() {
        let empty: [i32; 0] = [];
        smallest(&empty);
    }

    #[test]
    fn pair_exposes_first_and_second() {
        let pair = Pair::new(3, 7);
        assert_eq!(pair.first(), &3);
        assert_eq!(pair.second(), &7);
    }

    #[test]
    fn pair_larger_picks_the_bigger_value() {
        assert_eq!(Pair::new(3, 7).larger(), &7);
        assert_eq!(Pair::new(7, 3).larger(), &7);
    }

    #[test]
    fn pair_larger_breaks_a_tie_toward_first() {
        assert_eq!(Pair::new(5, 5).larger(), &5);
    }

    #[test]
    fn pair_works_with_non_numeric_types() {
        let pair = Pair::new(String::from("a"), String::from("b"));
        assert_eq!(pair.larger(), &String::from("b"));
    }

    #[test]
    fn matches_count_counts_predicate_hits() {
        assert_eq!(matches_count(&[1, 2, 3, 4, 5], |&n| n % 2 == 0), 2);
    }

    #[test]
    fn matches_count_of_empty_slice_is_zero() {
        let empty: [i32; 0] = [];
        assert_eq!(matches_count(&empty, |&n| n > 0), 0);
    }

    #[test]
    fn matches_count_works_on_strings() {
        let words = ["ok", "no", "okay"];
        assert_eq!(matches_count(&words, |w| w.starts_with('o')), 2);
    }

    #[test]
    fn label_and_duplicate_builds_the_exact_label() {
        let (label, kept) = label_and_duplicate(5);
        assert_eq!(label, "label: 5");
        assert_eq!(kept, 5);
    }

    #[test]
    fn label_and_duplicate_works_on_strings() {
        let (label, kept) = label_and_duplicate(String::from("Frieren"));
        assert_eq!(label, "label: Frieren");
        assert_eq!(kept, "Frieren");
    }
}
