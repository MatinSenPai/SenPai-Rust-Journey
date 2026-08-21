/// Takes ownership of both `first` and `second`, joins them with a space,
/// and returns a new, uppercased `String`.
pub fn combine_and_shout(first: String, second: String) -> String {
    todo!("format!(\"{{first}} {{second}}\").to_uppercase(), roughly")
}

/// Takes ownership of `s`, appends `suffix` to it, and returns ownership of
/// the (now-extended) `s` back to the caller — a "move in, mutate, move
/// back out" pattern you'll see constantly in real Rust code (e.g. builder
/// APIs that take `mut self` and return `Self`).
pub fn reclaim_and_extend(mut s: String, suffix: &str) -> String {
    todo!("s.push_str(suffix), then return s")
}

/// Takes ownership of the whole `Vec<String>` and returns the total number
/// of characters across every string in it. The vec (and every `String`
/// inside it) is fully consumed — dropped at the end of this function.
pub fn total_length(strings: Vec<String>) -> usize {
    todo!("iterate `strings`, summing each String's .len()")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_and_shouts() {
        assert_eq!(
            combine_and_shout("hello".to_string(), "world".to_string()),
            "HELLO WORLD"
        );
    }

    #[test]
    fn reclaims_and_extends() {
        assert_eq!(reclaim_and_extend("rust".to_string(), "acean"), "rustacean");
    }

    #[test]
    fn totals_length() {
        let strings = vec!["a".to_string(), "bb".to_string(), "ccc".to_string()];
        assert_eq!(total_length(strings), 6);
    }

    #[test]
    fn totals_length_of_empty_vec() {
        assert_eq!(total_length(vec![]), 0);
    }
}

// UNCOMMENT ME (then run `cargo check -p p1-02-02-move-semantics`):
//
// fn moved_value_demo() {
//     let s = String::from("hello");
//     let s2 = s;
//     println!("{s}"); // <- this line is the problem. Read the error Rust gives you.
//     println!("{s2}");
// }
