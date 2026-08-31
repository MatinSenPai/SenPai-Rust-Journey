//! Reference solution for 2.7.4 — property testing with `proptest`, snapshot
//! testing with `insta`.

/// Encodes `flags` as a `String` the same length as `flags`: `'1'` for every
/// `true`, `'0'` for every `false`, in order.
pub fn encode_flags(flags: &[bool]) -> String {
    flags.iter().map(|&b| if b { '1' } else { '0' }).collect()
}

/// Inverts [`encode_flags`]: for a `String` built by `encode_flags`, returns
/// the same flags back, in the same order. (Only needs to handle strings
/// `encode_flags` could actually have produced.)
pub fn decode_flags(encoded: &str) -> Vec<bool> {
    encoded.chars().map(|c| c == '1').collect()
}

/// One line of a watch checklist: a title, and whether it's been watched.
pub struct WatchItem {
    pub title: String,
    pub watched: bool,
}

/// Formats `items` as a checklist, one line per item, in the given order:
/// `[x] {title}` when `watched` is `true`, `[ ] {title}` when it's `false`.
/// Every line — including the last — ends with `\n`.
pub fn checklist(items: &[WatchItem]) -> String {
    let mut out = String::new();
    for item in items {
        let mark = if item.watched { "x" } else { " " };
        out.push_str(&format!("[{mark}] {}\n", item.title));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        #[test]
        fn flags_round_trip(flags in prop::collection::vec(any::<bool>(), 0..16)) {
            prop_assert_eq!(decode_flags(&encode_flags(&flags)), flags);
        }
    }

    #[test]
    fn encode_flags_matches_the_spec_on_one_hand_picked_case() {
        assert_eq!(encode_flags(&[true, false, true]), "101");
    }

    #[test]
    fn checklist_snapshot() {
        let items = vec![
            WatchItem {
                title: "Frieren".to_string(),
                watched: true,
            },
            WatchItem {
                title: "Bocchi the Rock!".to_string(),
                watched: true,
            },
            WatchItem {
                title: "Dandadan".to_string(),
                watched: false,
            },
        ];
        insta::assert_snapshot!(checklist(&items));
    }
}
