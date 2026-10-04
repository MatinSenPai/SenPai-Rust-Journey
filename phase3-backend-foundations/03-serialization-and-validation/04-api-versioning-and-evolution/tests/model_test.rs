//! The pure logic: conversions, the classifier, the headers, negotiation.

use p3_03_04_api_versioning_and_evolution::*;

fn frieren() -> Anime {
    Anime {
        id: 1,
        title: "Frieren".into(),
        status: WatchStatus::Watching,
        rating: Some(9),
        episodes: Some(28),
    }
}

#[test]
fn v1_has_the_frozen_shape_and_no_episodes() {
    let json = serde_json::to_string(&AnimeV1::from(&frieren())).unwrap();
    assert_eq!(
        json,
        r#"{"id":1,"title":"Frieren","status":"watching","rating":9}"#
    );
}

#[test]
fn v2_renames_status_and_adds_episodes() {
    let json = serde_json::to_string(&AnimeV2::from(&frieren())).unwrap();
    assert_eq!(
        json,
        r#"{"id":1,"title":"Frieren","watch_status":"watching","rating":9,"episodes":28}"#
    );
}

#[test]
fn v2_omits_unknown_episodes_but_keeps_a_null_rating() {
    let a = Anime {
        rating: None,
        episodes: None,
        ..frieren()
    };
    let json = serde_json::to_string(&AnimeV2::from(&a)).unwrap();
    assert_eq!(
        json,
        r#"{"id":1,"title":"Frieren","watch_status":"watching","rating":null}"#
    );
}

#[test]
fn v1_keeps_a_null_rating() {
    let a = Anime {
        rating: None,
        ..frieren()
    };
    let json = serde_json::to_string(&AnimeV1::from(&a)).unwrap();
    assert!(json.ends_with(r#""rating":null}"#), "{json}");
}

#[test]
fn every_change_is_classified_as_the_table_says() {
    use Change::*;
    use Compat::*;
    let table = [
        (AddResponseField, Compatible),
        (RemoveResponseField, Breaking),
        (RenameResponseField, Breaking),
        (ChangeResponseFieldType, Breaking),
        (AddResponseEnumValue, Breaking),
        (RemoveResponseEnumValue, Compatible),
        (AddOptionalRequestField, Compatible),
        (AddRequiredRequestField, Breaking),
        (RemoveRequestField, Compatible),
        (TightenRequestValidation, Breaking),
        (LoosenRequestValidation, Compatible),
        (AddEndpoint, Compatible),
        (RemoveEndpoint, Breaking),
    ];
    for (change, expected) in table {
        assert_eq!(classify(change), expected, "{change:?}");
    }
}

#[test]
fn a_release_needs_a_new_version_only_if_something_breaks() {
    use Change::*;
    assert!(!requires_new_version(&[]));
    assert!(!requires_new_version(&[AddResponseField, AddEndpoint]));
    assert!(requires_new_version(&[
        AddResponseField,
        RenameResponseField
    ]));
    assert!(requires_new_version(&[TightenRequestValidation]));
}

#[test]
fn deprecation_headers_are_three_pairs_in_order() {
    let h = deprecation_headers(1_767_225_600, "Thu, 31 Dec 2026 23:59:59 GMT", "/v2/anime");
    assert_eq!(
        h,
        vec![
            ("deprecation", "@1767225600".to_string()),
            ("sunset", "Thu, 31 Dec 2026 23:59:59 GMT".to_string()),
            ("link", "</v2/anime>; rel=\"successor-version\"".to_string()),
        ]
    );
}

#[test]
fn accept_picks_the_first_known_vendor_type() {
    use ApiVersion::*;
    let cases = [
        (None, V1),
        (Some("*/*"), V1),
        (Some("application/json"), V1),
        (Some("application/vnd.anime.v1+json"), V1),
        (Some("application/vnd.anime.v2+json"), V2),
        (Some("APPLICATION/VND.ANIME.V2+JSON"), V2),
        (Some("application/vnd.anime.v2+json; q=0.5"), V2),
        (Some("text/html, application/vnd.anime.v2+json"), V2),
        (
            Some("application/vnd.anime.v1+json, application/vnd.anime.v2+json"),
            V1,
        ),
        (
            Some("application/vnd.anime.v2+json, application/vnd.anime.v1+json"),
            V2,
        ),
        (Some("application/vnd.anime.v3+json"), V1),
    ];
    for (accept, expected) in cases {
        assert_eq!(version_from_accept(accept), expected, "{accept:?}");
    }
}
