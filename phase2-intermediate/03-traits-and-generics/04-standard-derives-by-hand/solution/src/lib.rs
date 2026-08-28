//! Solution for 2.3.4 — the standard derives, implemented by hand.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A single music track. Two `Track`s count as the same track when their
/// `title` and `artist` match — `seconds` is a measurement, not identity.
pub struct Track {
    pub title: String,
    pub artist: String,
    pub seconds: u32,
}

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Track")
            .field("title", &self.title)
            .field("artist", &self.artist)
            .field("seconds", &self.seconds)
            .finish()
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} by {}", self.title, self.artist)
    }
}

impl Default for Track {
    fn default() -> Self {
        Track {
            title: String::new(),
            artist: String::new(),
            seconds: 0,
        }
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.artist == other.artist
    }
}
impl Eq for Track {}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds
            .cmp(&other.seconds)
            .then_with(|| self.title.cmp(&other.title))
    }
}

impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.artist.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn queen() -> Track {
        Track {
            title: "Bohemian Rhapsody".to_string(),
            artist: "Queen".to_string(),
            seconds: 354,
        }
    }

    #[test]
    fn debug_matches_the_derive_shape() {
        assert_eq!(
            format!("{:?}", queen()),
            "Track { title: \"Bohemian Rhapsody\", artist: \"Queen\", seconds: 354 }"
        );
    }

    #[test]
    fn debug_alternate_form_is_indented() {
        let pretty = format!("{:#?}", queen());
        assert!(pretty.starts_with("Track {\n"));
        assert!(pretty.contains("    title: \"Bohemian Rhapsody\",\n"));
    }

    #[test]
    fn display_is_title_by_artist() {
        assert_eq!(format!("{}", queen()), "Bohemian Rhapsody by Queen");
    }

    #[test]
    fn default_is_all_blank() {
        let t = Track::default();
        assert_eq!(t.title, "");
        assert_eq!(t.artist, "");
        assert_eq!(t.seconds, 0);
    }

    #[test]
    fn eq_ignores_seconds() {
        let a = queen();
        let b = Track {
            seconds: 999,
            ..queen()
        };
        assert!(a == b);
    }

    #[test]
    fn eq_cares_about_title_and_artist() {
        let a = queen();
        let mut c = queen();
        c.artist = "Panic! at the Disco".to_string();
        assert!(a != c);
    }

    #[test]
    fn ord_sorts_by_seconds_then_title() {
        let mut tracks = vec![
            Track {
                title: "B".into(),
                artist: "x".into(),
                seconds: 200,
            },
            Track {
                title: "A".into(),
                artist: "x".into(),
                seconds: 200,
            },
            Track {
                title: "Z".into(),
                artist: "x".into(),
                seconds: 100,
            },
        ];
        tracks.sort();
        let order: Vec<&str> = tracks.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(order, vec!["Z", "A", "B"]);
    }

    #[test]
    fn hash_matches_eq_so_a_hashset_dedupes() {
        let mut set: HashSet<Track> = HashSet::new();
        set.insert(queen());
        let same_song_different_timestamp = Track {
            seconds: 1,
            ..queen()
        };
        let inserted_again = set.insert(same_song_different_timestamp);
        assert!(!inserted_again);
        assert_eq!(set.len(), 1);
    }
}
