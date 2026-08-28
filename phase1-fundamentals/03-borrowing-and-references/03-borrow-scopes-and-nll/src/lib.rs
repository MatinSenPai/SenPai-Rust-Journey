//! Exercises for 1.3.3 — borrow scopes and NLL.
//!
//! Every one of these reads a value and then changes the same value. The
//! borrow checker allows that; what it will not allow is the read still being
//! alive when the change happens. Where you put the read is the exercise.

/// `values` with a copy of its first element added on the end.
///
/// An empty `Vec` comes back empty.
///
/// # Examples
///
/// `with_first_repeated(vec![3, 5])` returns `[3, 5, 3]`.
/// `with_first_repeated(vec![7])` returns `[7, 7]`.
/// `with_first_repeated(vec![])` returns `[]`.
pub fn with_first_repeated(values: Vec<i32>) -> Vec<i32> {
    todo!("put a copy of the first element on the end; leave an empty Vec alone")
}

/// `values` with the sum of everything in it added on the end.
///
/// The sum of nothing is zero, so an empty `Vec` comes back holding one `0`.
///
/// # Examples
///
/// `with_total_appended(vec![1, 2, 3])` returns `[1, 2, 3, 6]`.
/// `with_total_appended(vec![-2, 2])` returns `[-2, 2, 0]`.
/// `with_total_appended(vec![])` returns `[0]`.
pub fn with_total_appended(values: Vec<i32>) -> Vec<i32> {
    todo!("add up what is already there, then put the total on the end")
}

/// `names` with a second copy of its longest string added on the end.
///
/// Length is `String::len` — bytes, the same measure as in 1.1.6. When two
/// names are the same length the earlier one wins. An empty `Vec` comes back
/// empty.
///
/// # Examples
///
/// `with_longest_repeated(vec!["ab".to_string(), "c".to_string()])` returns
/// `["ab", "c", "ab"]`.
/// `with_longest_repeated(vec!["a".to_string(), "b".to_string()])` returns
/// `["a", "b", "a"]`.
/// `with_longest_repeated(vec![])` returns `[]`.
pub fn with_longest_repeated(names: Vec<String>) -> Vec<String> {
    todo!("find the longest name and put a second copy of it on the end")
}

/// `text` with its own byte length written on the end in decimal digits.
///
/// The length is the one `text` had *before* anything was added.
///
/// # Examples
///
/// `with_length_appended("hi".to_string())` returns `"hi2"`.
/// `with_length_appended("hello".to_string())` returns `"hello5"`.
/// `with_length_appended("".to_string())` returns `"0"`.
pub fn with_length_appended(text: String) -> String {
    todo!("write the string's own byte length, in digits, on the end of it")
}

/// Every element of `values` doubled, with the sum of the doubled elements
/// added on the end.
///
/// # Examples
///
/// `doubled_then_totalled(vec![1, 2, 3])` returns `[2, 4, 6, 12]`.
/// `doubled_then_totalled(vec![5])` returns `[10, 10]`.
/// `doubled_then_totalled(vec![])` returns `[0]`.
pub fn doubled_then_totalled(values: Vec<i32>) -> Vec<i32> {
    todo!("double each element where it sits, then put the new total on the end")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_the_first_element() {
        assert_eq!(with_first_repeated(vec![3, 5]), vec![3, 5, 3]);
        assert_eq!(with_first_repeated(vec![7]), vec![7, 7]);
        assert_eq!(with_first_repeated(vec![]), Vec::<i32>::new());
    }

    #[test]
    fn appends_the_total() {
        assert_eq!(with_total_appended(vec![1, 2, 3]), vec![1, 2, 3, 6]);
        assert_eq!(with_total_appended(vec![-2, 2]), vec![-2, 2, 0]);
        assert_eq!(with_total_appended(vec![]), vec![0]);
    }

    #[test]
    fn repeats_the_longest_name() {
        assert_eq!(
            with_longest_repeated(vec!["ab".to_string(), "c".to_string()]),
            vec!["ab", "c", "ab"]
        );
        assert_eq!(
            with_longest_repeated(vec!["a".to_string(), "b".to_string()]),
            vec!["a", "b", "a"],
            "on a tie the earlier name wins"
        );
        assert_eq!(with_longest_repeated(vec![]), Vec::<String>::new());
    }

    #[test]
    fn appends_the_length_in_digits() {
        assert_eq!(with_length_appended("hi".to_string()), "hi2");
        assert_eq!(with_length_appended("hello".to_string()), "hello5");
        assert_eq!(with_length_appended("".to_string()), "0");
    }

    #[test]
    fn doubles_then_totals() {
        assert_eq!(doubled_then_totalled(vec![1, 2, 3]), vec![2, 4, 6, 12]);
        assert_eq!(doubled_then_totalled(vec![5]), vec![10, 10]);
        assert_eq!(doubled_then_totalled(vec![]), vec![0]);
    }
}
