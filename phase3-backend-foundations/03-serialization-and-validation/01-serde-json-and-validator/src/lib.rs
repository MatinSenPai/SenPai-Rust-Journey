use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

/// A review a user is submitting for an anime. `#[derive(Deserialize,
/// Serialize)]` handles *shape* (JSON <-> struct); `#[derive(Validate)]`
/// handles *rules* (is the shape-correct data actually acceptable) — see
/// the README for why these are deliberately two separate passes.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ReviewSubmission {
    #[validate(length(min = 1, max = 200, message = "title must be 1-200 characters"))]
    pub title: String,

    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    pub rating: u8,

    /// Absent in the incoming JSON entirely -> `None` (`#[serde(default)]`),
    /// not a deserialization error. `None` when serializing back out ->
    /// the key is omitted entirely (`skip_serializing_if`), not written as
    /// `"comment": null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000, message = "comment must be at most 1000 characters"))]
    pub comment: Option<String>,
}

/// Everything that can go wrong turning a raw JSON string into a validated
/// `ReviewSubmission` — kept as two distinct variants (not one generic
/// "bad request") for the same reason `HttpParseError` was, back in
/// module 1: a caller should be able to tell "this wasn't even JSON" apart
/// from "this was JSON, but broke a validation rule."
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReviewError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("validation failed: {0:?}")]
    Invalid(Vec<String>),
}

/// Flattens `validator`'s `ValidationErrors` (a `HashMap<&str,
/// &Vec<ValidationError>>` under the hood) into a sorted, human-readable
/// `Vec<String>` — one entry per broken rule, formatted `"field: message"`.
/// Sorted so callers (and tests) get a deterministic order regardless of
/// the underlying `HashMap`'s iteration order.
pub fn validation_summary(errors: &ValidationErrors) -> Vec<String> {
    todo!(
        "errors.field_errors() gives a HashMap<&str, &Vec<ValidationError>>; for each (field, \
         errs) pair, for each ValidationError `e` in errs, build a String: use \
         e.message.clone() if Some(_), else fall back to e.code.clone() — both are Cow<str>, \
         .to_string() either — formatted as \"{{field}}: {{message}}\"; collect every one of \
         those into a Vec<String>, .sort() it, and return it"
    )
}

/// Deserializes `json` into a `ReviewSubmission`, then validates it.
/// Returns `Err(ReviewError::InvalidJson(_))` if `json` doesn't even parse
/// into the right shape, or `Err(ReviewError::Invalid(_))` (via
/// `validation_summary`) if it parses but breaks a validation rule.
pub fn parse_review(json: &str) -> Result<ReviewSubmission, ReviewError> {
    todo!(
        "serde_json::from_str::<ReviewSubmission>(json), mapping any Err to \
         ReviewError::InvalidJson(e.to_string()) with `?`; then call .validate() on the result, \
         mapping any Err(errors) through validation_summary(&errors) into \
         ReviewError::Invalid(...) with `?`; finally return Ok(submission)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_a_well_formed_review() {
        let json = r#"{"title": "Peak fiction", "rating": 10, "comment": "no notes"}"#;
        let review = parse_review(json).unwrap();
        assert_eq!(review.title, "Peak fiction");
        assert_eq!(review.rating, 10);
        assert_eq!(review.comment.as_deref(), Some("no notes"));
    }

    #[test]
    fn comment_defaults_to_none_when_absent() {
        let json = r#"{"title": "Fine, I guess", "rating": 6}"#;
        let review = parse_review(json).unwrap();
        assert_eq!(review.comment, None);
    }

    #[test]
    fn rejects_malformed_json_as_invalid_json_not_invalid_data() {
        let result = parse_review("{ not json at all");
        assert!(matches!(result, Err(ReviewError::InvalidJson(_))));
    }

    #[test]
    fn rejects_a_missing_required_field_as_invalid_json() {
        // `title` is required (no `#[serde(default)]`), so a JSON object
        // missing it entirely fails to deserialize at all — this is a
        // *shape* problem, not a validation-rule problem.
        let result = parse_review(r#"{"rating": 5}"#);
        assert!(matches!(result, Err(ReviewError::InvalidJson(_))));
    }

    #[test]
    fn rejects_an_out_of_range_rating() {
        let json = r#"{"title": "Mid", "rating": 11}"#;
        let result = parse_review(json);
        match result {
            Err(ReviewError::Invalid(messages)) => {
                assert!(messages.iter().any(|m| m.starts_with("rating:")));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn rejects_an_empty_title() {
        let json = r#"{"title": "", "rating": 5}"#;
        let result = parse_review(json);
        match result {
            Err(ReviewError::Invalid(messages)) => {
                assert!(messages.iter().any(|m| m.starts_with("title:")));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn reports_every_broken_rule_at_once_sorted_by_field() {
        let json = r#"{"title": "", "rating": 0}"#;
        let result = parse_review(json);
        match result {
            Err(ReviewError::Invalid(messages)) => {
                assert_eq!(messages.len(), 2);
                assert!(messages[0].starts_with("rating:"));
                assert!(messages[1].starts_with("title:"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn serializing_omits_a_missing_comment_entirely() {
        let review = ReviewSubmission {
            title: "Solid".to_string(),
            rating: 8,
            comment: None,
        };
        let value = serde_json::to_value(&review).unwrap();
        assert!(value.get("comment").is_none());
    }

    #[test]
    fn serializing_includes_a_present_comment() {
        let review = ReviewSubmission {
            title: "Solid".to_string(),
            rating: 8,
            comment: Some("would watch again".to_string()),
        };
        let value = serde_json::to_value(&review).unwrap();
        assert_eq!(value["comment"], "would watch again");
    }
}
