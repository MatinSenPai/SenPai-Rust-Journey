//! DELIBERATELY BROKEN — expected: a run-time panic — insta's snapshot
//! assertion fails because the committed `.snap` predates a wording change
//! in the code below ("episodes" -> "eps").
//!
//!     cargo test -p p2-07-04-property-and-snapshot-testing --example 04-broken-stale-snapshot --features broken

struct Entry {
    title: String,
    episodes_watched: u32,
    rating: Option<u8>,
}

fn sample_entries() -> Vec<Entry> {
    vec![
        Entry {
            title: "Frieren".to_string(),
            episodes_watched: 12,
            rating: Some(9),
        },
        Entry {
            title: "Bocchi the Rock!".to_string(),
            episodes_watched: 12,
            rating: Some(10),
        },
        Entry {
            title: "Made in Abyss".to_string(),
            episodes_watched: 3,
            rating: None,
        },
    ]
}

fn watch_digest(entries: &[Entry]) -> String {
    let mut out = String::new();
    for e in entries {
        let rating = match e.rating {
            Some(r) => format!("{r}/10"),
            None => "unrated".to_string(),
        };
        // Someone shortened "episodes" to "eps" here without telling anyone.
        // The committed snapshot still says "episodes".
        out.push_str(&format!(
            "{} - {} eps - {}\n",
            e.title, e.episodes_watched, rating
        ));
    }
    out
}

fn main() {
    print!("{}", watch_digest(&sample_entries()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_snapshot() {
        insta::assert_snapshot!(watch_digest(&sample_entries()));
    }
}
