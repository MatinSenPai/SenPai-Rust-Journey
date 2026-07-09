pub fn find_by_id(users: &[(u32, String)], id: u32) -> Option<String> {
    users
        .iter()
        .find(|(uid, _)| *uid == id)
        .map(|(_, name)| name.clone())
}

pub fn describe_lookup(result: Option<String>) -> String {
    match result {
        Some(name) => format!("Found: {name}"),
        None => "User not found".to_string(),
    }
}

pub fn average_known_age(ages: &[Option<u32>]) -> Option<f64> {
    let known: Vec<u32> = ages.iter().filter_map(|a| *a).collect();
    if known.is_empty() {
        return None;
    }
    let sum: u32 = known.iter().sum();
    Some(sum as f64 / known.len() as f64)
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
