//! The rules and the error flattening, with plain function calls: no
//! `axum`, no runtime, no requests.

use p3_03_02_validation::{flatten_errors, validate_handle, EpisodeNote, NewReview, Reviewer};
use validator::Validate;

fn good_reviewer() -> Reviewer {
    Reviewer {
        handle: "matin_01".into(),
        email: "matin@example.com".into(),
    }
}

fn good_review() -> NewReview {
    NewReview {
        title: "Frieren".into(),
        rating: 9,
        body: None,
        reviewer: good_reviewer(),
        notes: vec![],
    }
}

fn flat(review: &NewReview) -> Vec<(String, Vec<String>)> {
    match review.validate() {
        Ok(()) => vec![],
        Err(e) => flatten_errors(&e).into_iter().collect(),
    }
}

#[test]
fn handle_accepts_letters_digits_and_underscore() {
    assert!(validate_handle("Matin_01").is_ok());
    assert!(validate_handle("").is_ok());
}

#[test]
fn handle_rejects_anything_else_with_the_exact_code_and_message() {
    for bad in ["a b", "a-b", "سلام", "a!"] {
        let err = validate_handle(bad).unwrap_err();
        assert_eq!(err.code, "handle_chars", "handle {bad:?}");
        assert_eq!(
            err.message.as_deref(),
            Some("handle may only contain letters, digits and underscores")
        );
    }
}

#[test]
fn a_good_review_has_no_errors() {
    assert!(good_review().validate().is_ok());
    assert!(flat(&good_review()).is_empty());
}

#[test]
fn a_field_error_is_keyed_by_the_field_name() {
    let mut r = good_review();
    r.rating = 15;
    assert_eq!(
        flat(&r),
        vec![(
            "rating".to_string(),
            vec!["rating must be between 1 and 10".to_string()]
        )]
    );
}

#[test]
fn every_broken_field_is_reported_at_once_sorted_by_key() {
    let mut r = good_review();
    r.title = String::new();
    r.rating = 0;
    let keys: Vec<String> = flat(&r).into_iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["rating", "title"]);
}

#[test]
fn a_rule_without_a_message_reports_its_code() {
    let mut r = good_review();
    r.reviewer.email = "not-an-email".into();
    assert_eq!(
        flat(&r),
        vec![("reviewer.email".to_string(), vec!["email".to_string()])]
    );
}

#[test]
fn a_nested_struct_prefixes_the_parent_field() {
    let mut r = good_review();
    r.reviewer.handle = "x".into();
    assert_eq!(
        flat(&r),
        vec![(
            "reviewer.handle".to_string(),
            vec!["handle must be 3 to 20 characters".to_string()]
        )]
    );
}

#[test]
fn two_rules_on_one_field_give_two_messages_sorted() {
    let mut r = good_review();
    r.reviewer.handle = "a!".into();
    assert_eq!(
        flat(&r),
        vec![(
            "reviewer.handle".to_string(),
            vec![
                "handle may only contain letters, digits and underscores".to_string(),
                "handle must be 3 to 20 characters".to_string(),
            ]
        )]
    );
}

#[test]
fn a_failing_list_item_is_keyed_by_its_index() {
    let mut r = good_review();
    r.notes = vec![
        EpisodeNote {
            episode: 1,
            text: "fine".into(),
        },
        EpisodeNote {
            episode: 0,
            text: String::new(),
        },
    ];
    assert_eq!(
        flat(&r),
        vec![
            ("notes[1].episode".to_string(), vec!["range".to_string()]),
            (
                "notes[1].text".to_string(),
                vec!["note must be 1 to 200 characters".to_string()]
            ),
        ]
    );
}

#[test]
fn an_empty_error_tree_flattens_to_an_empty_map() {
    assert!(flatten_errors(&validator::ValidationErrors::new()).is_empty());
}
