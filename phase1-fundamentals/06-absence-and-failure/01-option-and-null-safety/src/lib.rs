/// Searches `users` (a slice of `(id, name)` pairs) for `id`, returning a
/// clone of the matching name, or `None` if no user has that id.
pub fn find_by_id(users: &[(u32, String)], id: u32) -> Option<String> {
    todo!("users.iter().find(|(uid, _)| *uid == id).map(|(_, name)| name.clone())")
}

/// Turns a lookup result into a message: `"Found: Matin"` if `Some`, or
/// `"User not found"` if `None`.
pub fn describe_lookup(result: Option<String>) -> String {
    todo!("result.map(|name| format!(\"Found: {{name}}\")).unwrap_or_else(...) — or match/if let")
}

/// Averages only the `Some` values in `ages`, ignoring every `None`.
/// Returns `None` if there are zero `Some` values (avoid dividing by zero).
pub fn average_known_age(ages: &[Option<u32>]) -> Option<f64> {
    todo!("filter_map to get the known ages, then average them, or None if empty")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_users() -> Vec<(u32, String)> {
        vec![
            (1, "Matin".to_string()),
            (2, "Yui".to_string()),
            (3, "Levi".to_string()),
        ]
    }

    #[test]
    fn finds_existing_user() {
        assert_eq!(find_by_id(&sample_users(), 2), Some("Yui".to_string()));
    }

    #[test]
    fn returns_none_for_missing_user() {
        assert_eq!(find_by_id(&sample_users(), 99), None);
    }

    #[test]
    fn describes_found_and_missing() {
        assert_eq!(describe_lookup(Some("Matin".to_string())), "Found: Matin");
        assert_eq!(describe_lookup(None), "User not found");
    }

    #[test]
    fn averages_known_ages_only() {
        assert_eq!(average_known_age(&[Some(20), None, Some(30)]), Some(25.0));
        assert_eq!(average_known_age(&[None, None]), None);
        assert_eq!(average_known_age(&[]), None);
    }
}
