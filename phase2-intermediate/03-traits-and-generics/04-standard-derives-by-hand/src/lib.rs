//! Exercises for 2.3.4 — the standard derives, implemented by hand.
//!
//! One `Track` type, six trait impls. `PartialOrd` and `Eq` are given to you
//! (they are one-line deliveries once `Ord` and `PartialEq` exist) — you
//! write the six pieces that actually decide something.

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

/// Formats `self` in exactly the shape `#[derive(Debug)]` would produce for
/// these three fields, in declaration order, and — because it is built the
/// way a derive is built — supports the alternate `{:#?}` form too.
///
/// # Examples
///
/// For `Track { title: "Bohemian Rhapsody".to_string(), artist:
/// "Queen".to_string(), seconds: 354 }`, `format!("{track:?}")` is exactly
/// `Track { title: "Bohemian Rhapsody", artist: "Queen", seconds: 354 }`.
impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("produce the exact Debug text documented above, including support for the alternate pretty-printed form")
    }
}

/// Formats `self` for a listener, not a debugger: the title, the word
/// "by", and the artist — nothing else, no quotes.
///
/// # Examples
///
/// For the same track as above, `format!("{track}")` is exactly
/// `Bohemian Rhapsody by Queen`.
impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("produce the exact Display text documented above")
    }
}

/// A blank starting point: empty `title`, empty `artist`, `seconds` zero.
impl Default for Track {
    fn default() -> Self {
        todo!("build a Track with an empty title, an empty artist, and zero seconds")
    }
}

/// Two tracks are equal exactly when their `title` and `artist` match.
/// `seconds` is never part of the comparison — two recordings of the same
/// song still count as the same track even if their tracked runtime
/// differs.
impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        todo!("compare title and artist only; ignore seconds entirely")
    }
}
impl Eq for Track {}

impl PartialOrd for Track {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders tracks by `seconds` ascending (shortest first). When two tracks
/// have the same `seconds`, the tie is broken by `title`, using `String`'s
/// own ordering (`Ord` on `str`/`String` is byte-value order).
impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!("compare by seconds first; if that is equal, fall back to comparing title")
    }
}

/// Hashes exactly the fields `PartialEq` compares — `title` and `artist` —
/// and nothing else. `seconds` must stay out of this impl: `PartialEq`
/// above already decided it doesn't affect identity, and a hasher that
/// disagrees breaks every `HashSet`/`HashMap` that uses `Track` as a key.
impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!("feed title and artist into state, in that order; do not touch seconds")
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
