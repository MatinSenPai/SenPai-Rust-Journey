//! Reference solution for 1.2.3 — `Clone` and `Copy`.
//!
//! Two of these need a clone. Two of them do not and would still compile if
//! you added one. Knowing which is which is the lesson.

/// `text`, and a second independent copy of it.
///
/// Changing one afterwards must not affect the other.
///
/// # Examples
///
/// `duplicated("hi".to_string())` returns `("hi", "hi")` as two separate
/// `String`s.
pub fn duplicated(text: String) -> (String, String) {
    let copy = text.clone();
    (text, copy)
}

/// `values` unchanged, together with the sum of everything in it.
///
/// The array comes back as well as the sum. That is possible without any
/// cloning, and working out why is the point of this one.
///
/// # Examples
///
/// `array_survives([1, 2, 3, 4])` returns `([1, 2, 3, 4], 10)`.
/// `array_survives([0, 0, 0, 0])` returns `([0, 0, 0, 0], 0)`.
pub fn array_survives(values: [i32; 4]) -> ([i32; 4], i32) {
    let mut total = 0;
    for value in values {
        total += value;
    }
    (values, total)
}

/// `values` with no reserved room going spare: its capacity ends up equal to
/// its length.
///
/// The contents are unchanged.
///
/// # Examples
///
/// Given a `Vec` with three items and capacity 100, the answer has three
/// items and capacity 3.
pub fn shrunk(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    values.shrink_to_fit();
    values
}

/// Every string in `values`, each one appearing twice in a row.
///
/// # Examples
///
/// `doubled_up(vec!["a".to_string(), "b".to_string()])` returns
/// `["a", "a", "b", "b"]`.
/// `doubled_up(vec![])` returns `[]`.
pub fn doubled_up(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(values.len() * 2);
    for value in values {
        out.push(value.clone());
        out.push(value);
    }
    out
}

/// `text` repeated `times` times, as separate `String`s.
///
/// `repeated(t, 0)` is empty. Note that the last one out can be `text`
/// itself — you need one fewer clone than you might first think.
///
/// # Examples
///
/// `repeated("ab".to_string(), 3)` returns `["ab", "ab", "ab"]`.
/// `repeated("ab".to_string(), 1)` returns `["ab"]`.
/// `repeated("ab".to_string(), 0)` returns `[]`.
pub fn repeated(text: String, times: usize) -> Vec<String> {
    if times == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(times);
    for _ in 1..times {
        out.push(text.clone());
    }
    out.push(text);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_two_independent_strings() {
        let (mut left, right) = duplicated("hi".to_string());
        assert_eq!(left, "hi");
        assert_eq!(right, "hi");
        left.push('!');
        assert_eq!(left, "hi!");
        assert_eq!(right, "hi", "the two must not share a buffer");
    }

    #[test]
    fn the_array_comes_back_too() {
        assert_eq!(array_survives([1, 2, 3, 4]), ([1, 2, 3, 4], 10));
        assert_eq!(array_survives([0, 0, 0, 0]), ([0, 0, 0, 0], 0));
        assert_eq!(array_survives([-1, 1, -1, 1]), ([-1, 1, -1, 1], 0));
    }

    #[test]
    fn removes_the_spare_room() {
        let mut roomy: Vec<i32> = Vec::with_capacity(100);
        roomy.push(1);
        roomy.push(2);
        roomy.push(3);

        let tight = shrunk(roomy);
        assert_eq!(tight, vec![1, 2, 3]);
        assert_eq!(tight.capacity(), tight.len());

        let empty = shrunk(Vec::new());
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn repeats_each_element_once() {
        assert_eq!(
            doubled_up(vec!["a".to_string(), "b".to_string()]),
            vec!["a", "a", "b", "b"]
        );
        assert_eq!(doubled_up(vec!["only".to_string()]), vec!["only", "only"]);
        assert_eq!(doubled_up(vec![]), Vec::<String>::new());
    }

    #[test]
    fn repeats_a_string_n_times() {
        assert_eq!(repeated("ab".to_string(), 3), vec!["ab", "ab", "ab"]);
        assert_eq!(repeated("ab".to_string(), 1), vec!["ab"]);
        assert_eq!(repeated("ab".to_string(), 0), Vec::<String>::new());

        // Each one must own its own buffer.
        let mut made = repeated("x".to_string(), 2);
        made[0].push('!');
        assert_eq!(made[0], "x!");
        assert_eq!(made[1], "x");
    }
}
