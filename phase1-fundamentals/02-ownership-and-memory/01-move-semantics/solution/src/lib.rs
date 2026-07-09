pub fn combine_and_shout(first: String, second: String) -> String {
    format!("{first} {second}").to_uppercase()
}

pub fn reclaim_and_extend(mut s: String, suffix: &str) -> String {
    s.push_str(suffix);
    s
}

pub fn total_length(strings: Vec<String>) -> usize {
    let mut total = 0;
    for s in strings {
        total += s.len();
    }
    total
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
