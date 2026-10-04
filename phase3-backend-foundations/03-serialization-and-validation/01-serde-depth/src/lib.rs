//! Exercises for 3.3.1 — Serde in depth.
//!
//! No HTTP, no framework: only `serde` and `serde_json`. Five of the types
//! below are "yours": their doc comments spell out the exact JSON they must
//! read and write, and it is on you to add the `#[serde(...)]` attributes
//! (and the function bodies) that make that true. The README walks through
//! every attribute you need.

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

/// Where you are with a show. Exactly as in 3.2.3: on the wire it is
/// snake_case — `"watching"`, `"completed"`, `"plan_to_watch"`, `"dropped"`.
/// Already complete; nothing to do here.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    #[default]
    PlanToWatch,
    Dropped,
}

/// The catalog entry from 3.2.3, already complete. As JSON:
/// `{"id":1,"title":"Frieren","status":"watching","rating":9}`, with
/// `"rating":null` when the show has no rating yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Anime {
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

// ---------------------------------------------------------------------------
// Implement 1: renaming, skipping and defaults
// ---------------------------------------------------------------------------

/// A compact card for a list screen.
///
/// **JSON shape** (compact, keys in exactly this order):
///
/// ```json
/// {"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"rating":9}
/// ```
///
/// - Keys are camelCase on the wire (`watchStatus`, `episodeCount`); the Rust
///   fields stay snake_case.
/// - `rating` is left out of the output entirely when it is `None`. When
///   reading, `rating` may be missing or `null`; both mean `None`.
/// - When reading, `episodeCount` may be missing and then means `0`.
/// - Keys this struct does not know (for example `"studio"`) are ignored when
///   reading.
///
/// The attributes that produce this are yours to add.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeCard {
    pub id: u64,
    pub title: String,
    pub watch_status: WatchStatus,
    pub episode_count: u32,
    pub rating: Option<u8>,
}

/// Writes `card` as the compact JSON text described on [`AnimeCard`].
pub fn card_to_json(card: &AnimeCard) -> String {
    todo!("return the compact JSON text of the card, in the shape documented on AnimeCard")
}

/// Reads an [`AnimeCard`] from JSON text, following the reading rules on
/// [`AnimeCard`]. On failure the `Err` holds the `to_string()` of the
/// `serde_json` error, unchanged — for example a snake_case `watch_status`
/// key gives an error containing ``missing field `watchStatus` ``.
pub fn card_from_json(json: &str) -> Result<AnimeCard, String> {
    todo!("parse the JSON text into an AnimeCard, or return the parser's error text as the Err")
}

// ---------------------------------------------------------------------------
// Implement 2: flatten
// ---------------------------------------------------------------------------

/// When a record was created and last changed. Plain text timestamps; this
/// lesson does not parse them.
///
/// JSON keys: `createdAt` and `updatedAt`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: String,
    pub updated_at: String,
}

/// An [`AnimeCard`] plus its [`Audit`], as ONE flat JSON object — no nested
/// `"card"` or `"audit"` keys:
///
/// ```json
/// {"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"createdAt":"2026-01-05","updatedAt":"2026-02-01"}
/// ```
///
/// The card's keys come first, then the audit's, and the card's own rules
/// (a `None` rating is left out) still apply inside the flat object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeDetail {
    pub card: AnimeCard,
    pub audit: Audit,
}

/// Writes `detail` as the flat compact JSON text described on [`AnimeDetail`].
pub fn detail_to_json(detail: &AnimeDetail) -> String {
    todo!("return the compact flat JSON text of the detail, in the shape documented on AnimeDetail")
}

/// Reads an [`AnimeDetail`] from the flat JSON object described on
/// [`AnimeDetail`]. On failure the `Err` holds the `to_string()` of the
/// `serde_json` error, unchanged — a missing `updatedAt` key gives an error
/// containing ``missing field `updatedAt` ``.
pub fn detail_from_json(json: &str) -> Result<AnimeDetail, String> {
    todo!("parse the flat JSON text into an AnimeDetail, or return the parser's error text as the Err")
}

// ---------------------------------------------------------------------------
// Implement 3: deny_unknown_fields
// ---------------------------------------------------------------------------

/// The body a client sends to add a show:
///
/// ```json
/// {"title":"Frieren","status":"watching","rating":9}
/// ```
///
/// - `title` is required.
/// - `status` is optional; when it is missing it means
///   [`WatchStatus::PlanToWatch`].
/// - `rating` is optional; missing and `null` both mean `None`.
/// - Any other key is an error, so a typo such as `"ratng"` is reported
///   instead of silently dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAnime {
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

/// Reads a [`CreateAnime`] following the rules on that type. On failure the
/// `Err` holds the `to_string()` of the `serde_json` error, unchanged — for
/// example an unknown `ratng` key gives an error containing
/// ``unknown field `ratng` ``, and a missing title one containing
/// ``missing field `title` ``.
pub fn decode_create_anime(json: &str) -> Result<CreateAnime, String> {
    todo!("parse the JSON text into a CreateAnime following its rules, or return the parser's error text as the Err")
}

// ---------------------------------------------------------------------------
// Implement 4: tagged enums
// ---------------------------------------------------------------------------

/// Something that happened in the catalog. On the wire every event is an
/// object whose `"type"` key names the variant, and the variant's own fields
/// sit beside it:
///
/// ```json
/// {"type":"added","id":1,"title":"Frieren"}
/// {"type":"status_changed","id":1,"from":"watching","to":"completed"}
/// {"type":"removed","id":1}
/// ```
///
/// The `type` values are the snake_case variant names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimeEvent {
    Added {
        id: u64,
        title: String,
    },
    StatusChanged {
        id: u64,
        from: WatchStatus,
        to: WatchStatus,
    },
    Removed {
        id: u64,
    },
}

/// Writes `events` as one compact JSON array of the objects described on
/// [`AnimeEvent`], in order. An empty slice gives `[]`.
pub fn events_to_json(events: &[AnimeEvent]) -> String {
    todo!("return the compact JSON array text of the events, each in the shape documented on AnimeEvent")
}

/// Reads a JSON array of events. On failure the `Err` holds the `to_string()`
/// of the `serde_json` error, unchanged — an unknown `type` such as
/// `"renamed"` gives an error containing ``unknown variant `renamed` ``, and
/// an object with no `type` key one containing ``missing field `type` ``.
pub fn events_from_json(json: &str) -> Result<Vec<AnimeEvent>, String> {
    todo!("parse the JSON array text into events, or return the parser's error text as the Err")
}

// ---------------------------------------------------------------------------
// Implement 5: a hand-written Serialize
// ---------------------------------------------------------------------------

/// A rating from 1 to 10. On the wire it is a JSON **string** of the form
/// `"N/10"`: `Rating(9)` is `"9/10"`, `Rating(10)` is `"10/10"`.
///
/// Writing does no validation: `Rating(0)` is written as `"0/10"`. Reading
/// is the Build rung below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rating(pub u8);

impl Serialize for Rating {
    /// Writes the rating as the string described on [`Rating`].
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        todo!("write this rating as the string N/10 documented on Rating")
    }
}

// ---------------------------------------------------------------------------
// Build: a hand-written Deserialize with a Visitor
// ---------------------------------------------------------------------------

impl<'de> Deserialize<'de> for Rating {
    /// Reads a [`Rating`] from either of two JSON shapes:
    ///
    /// - a **string** `"N/10"` where `N` is an integer, e.g. `"9/10"`;
    /// - a bare **non-negative integer** `N`, e.g. `9`.
    ///
    /// In both shapes `N` must be `1..=10`. Any other `N` fails with exactly
    /// this message: `rating must be between 1 and 10, got N` (with the
    /// offending number in place of `N`; for `"0/10"` that is `got 0`).
    ///
    /// Any other input — a string that is not of the form `N/10`, a boolean,
    /// `null`, a negative integer, a float — is an error too. The
    /// visitor's `expecting` text must be exactly
    /// `a rating like "9/10" or an integer from 1 to 10`, so that for
    /// `true` the error reads
    /// ``invalid type: boolean `true`, expected a rating like "9/10" or an integer from 1 to 10 ``.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        todo!("read a Rating from a string N/10 or an integer N, following the rules in the doc comment")
    }
}

// ---------------------------------------------------------------------------
// Challenge (optional): serde(with = "...")
// ---------------------------------------------------------------------------

/// A show and its genres. On the wire `genres` is ONE string, comma
/// separated, not an array:
///
/// ```json
/// {"title":"Frieren","genres":"fantasy,adventure"}
/// ```
///
/// The `with` attribute below sends both directions through [`genre_list`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeGenres {
    pub title: String,
    #[serde(with = "genre_list")]
    pub genres: Vec<String>,
}

/// The two functions `#[serde(with = "genre_list")]` calls.
pub mod genre_list {
    use serde::{Deserializer, Serializer};

    /// Writes the genres as one string with `,` between them and nothing
    /// else added: `["fantasy","adventure"]` is `"fantasy,adventure"`, and
    /// an empty list is `""`.
    pub fn serialize<S: Serializer>(genres: &[String], serializer: S) -> Result<S::Ok, S::Error> {
        todo!("write the genres as one comma-separated string")
    }

    /// Reads one string back into genres: split on `,`, trim spaces around
    /// each piece, and drop pieces that end up empty. `"fantasy, adventure"`
    /// is `["fantasy","adventure"]`, and `""` is an empty list.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        todo!("read one comma-separated string into a list of genres, following the doc comment")
    }
}

#[cfg(test)]
mod card_tests {
    use super::*;

    fn frieren() -> AnimeCard {
        AnimeCard {
            id: 1,
            title: "Frieren".into(),
            watch_status: WatchStatus::Watching,
            episode_count: 28,
            rating: Some(9),
        }
    }

    #[test]
    fn card_is_written_in_camel_case() {
        assert_eq!(
            card_to_json(&frieren()),
            r#"{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"rating":9}"#
        );
    }

    #[test]
    fn a_missing_rating_is_left_out_not_null() {
        let card = AnimeCard {
            id: 2,
            title: "Mushishi".into(),
            watch_status: WatchStatus::Completed,
            episode_count: 26,
            rating: None,
        };
        assert_eq!(
            card_to_json(&card),
            r#"{"id":2,"title":"Mushishi","watchStatus":"completed","episodeCount":26}"#
        );
    }

    #[test]
    fn a_card_round_trips() {
        let json = card_to_json(&frieren());
        assert_eq!(card_from_json(&json), Ok(frieren()));
    }

    #[test]
    fn episode_count_and_rating_may_be_missing_when_reading() {
        let card = card_from_json(r#"{"id":3,"title":"X","watchStatus":"dropped"}"#).unwrap();
        assert_eq!(card.episode_count, 0);
        assert_eq!(card.rating, None);
    }

    #[test]
    fn a_null_rating_reads_as_none() {
        let json = r#"{"id":3,"title":"X","watchStatus":"dropped","rating":null}"#;
        assert_eq!(card_from_json(json).unwrap().rating, None);
    }

    #[test]
    fn unknown_keys_are_ignored_when_reading() {
        let json = r#"{"id":3,"title":"X","watchStatus":"dropped","studio":"Madhouse"}"#;
        assert!(card_from_json(json).is_ok());
    }

    #[test]
    fn a_snake_case_key_is_not_accepted() {
        let err = card_from_json(r#"{"id":3,"title":"X","watch_status":"dropped"}"#).unwrap_err();
        assert!(err.contains("missing field `watchStatus`"), "{err}");
    }

    #[test]
    fn anime_from_3_2_3_still_round_trips() {
        let a = Anime {
            id: 1,
            title: "Frieren".into(),
            status: WatchStatus::Watching,
            rating: None,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(
            json,
            r#"{"id":1,"title":"Frieren","status":"watching","rating":null}"#
        );
        assert_eq!(serde_json::from_str::<Anime>(&json).unwrap(), a);
    }
}

#[cfg(test)]
mod detail_tests {
    use super::*;

    fn detail() -> AnimeDetail {
        AnimeDetail {
            card: AnimeCard {
                id: 1,
                title: "Frieren".into(),
                watch_status: WatchStatus::Watching,
                episode_count: 28,
                rating: None,
            },
            audit: Audit {
                created_at: "2026-01-05".into(),
                updated_at: "2026-02-01".into(),
            },
        }
    }

    #[test]
    fn detail_is_one_flat_object() {
        assert_eq!(
            detail_to_json(&detail()),
            r#"{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"createdAt":"2026-01-05","updatedAt":"2026-02-01"}"#
        );
    }

    #[test]
    fn detail_round_trips() {
        let json = detail_to_json(&detail());
        assert_eq!(detail_from_json(&json), Ok(detail()));
    }

    #[test]
    fn a_missing_audit_key_is_reported() {
        let json = r#"{"id":1,"title":"F","watchStatus":"watching","createdAt":"a"}"#;
        let err = detail_from_json(json).unwrap_err();
        assert!(err.contains("missing field `updatedAt`"), "{err}");
    }

    #[test]
    fn a_nested_card_is_not_accepted() {
        let json = r#"{"card":{"id":1},"audit":{"createdAt":"a","updatedAt":"b"}}"#;
        assert!(detail_from_json(json).is_err());
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;

    #[test]
    fn only_the_title_is_required() {
        let c = decode_create_anime(r#"{"title":"Frieren"}"#).unwrap();
        assert_eq!(c.title, "Frieren");
        assert_eq!(c.status, WatchStatus::PlanToWatch);
        assert_eq!(c.rating, None);
    }

    #[test]
    fn every_field_is_read() {
        let c = decode_create_anime(r#"{"title":"F","status":"watching","rating":9}"#).unwrap();
        assert_eq!(c.status, WatchStatus::Watching);
        assert_eq!(c.rating, Some(9));
    }

    #[test]
    fn a_null_rating_is_none() {
        let c = decode_create_anime(r#"{"title":"F","rating":null}"#).unwrap();
        assert_eq!(c.rating, None);
    }

    #[test]
    fn a_typo_is_an_error_not_a_silent_drop() {
        let err = decode_create_anime(r#"{"title":"F","ratng":9}"#).unwrap_err();
        assert!(err.contains("unknown field `ratng`"), "{err}");
    }

    #[test]
    fn a_missing_title_is_an_error() {
        let err = decode_create_anime(r#"{"status":"dropped"}"#).unwrap_err();
        assert!(err.contains("missing field `title`"), "{err}");
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;

    fn events() -> Vec<AnimeEvent> {
        vec![
            AnimeEvent::Added {
                id: 1,
                title: "Frieren".into(),
            },
            AnimeEvent::StatusChanged {
                id: 1,
                from: WatchStatus::Watching,
                to: WatchStatus::Completed,
            },
            AnimeEvent::Removed { id: 1 },
        ]
    }

    #[test]
    fn events_carry_their_type_beside_their_fields() {
        assert_eq!(
            events_to_json(&events()),
            concat!(
                r#"[{"type":"added","id":1,"title":"Frieren"},"#,
                r#"{"type":"status_changed","id":1,"from":"watching","to":"completed"},"#,
                r#"{"type":"removed","id":1}]"#
            )
        );
    }

    #[test]
    fn no_events_is_an_empty_array() {
        assert_eq!(events_to_json(&[]), "[]");
    }

    #[test]
    fn events_round_trip() {
        let json = events_to_json(&events());
        assert_eq!(events_from_json(&json), Ok(events()));
    }

    #[test]
    fn an_unknown_type_is_named_in_the_error() {
        let err = events_from_json(r#"[{"type":"renamed","id":1}]"#).unwrap_err();
        assert!(err.contains("unknown variant `renamed`"), "{err}");
    }

    #[test]
    fn a_missing_type_is_named_in_the_error() {
        let err = events_from_json(r#"[{"id":1}]"#).unwrap_err();
        assert!(err.contains("missing field `type`"), "{err}");
    }
}

#[cfg(test)]
mod rating_write_tests {
    use super::*;

    #[test]
    fn a_rating_is_written_as_n_over_10() {
        assert_eq!(serde_json::to_string(&Rating(9)).unwrap(), r#""9/10""#);
        assert_eq!(serde_json::to_string(&Rating(10)).unwrap(), r#""10/10""#);
    }

    #[test]
    fn writing_does_not_validate() {
        assert_eq!(serde_json::to_string(&Rating(0)).unwrap(), r#""0/10""#);
    }

    #[test]
    fn a_rating_inside_other_data_is_written_the_same_way() {
        let v = vec![Rating(1), Rating(10)];
        assert_eq!(serde_json::to_string(&v).unwrap(), r#"["1/10","10/10"]"#);
    }
}
