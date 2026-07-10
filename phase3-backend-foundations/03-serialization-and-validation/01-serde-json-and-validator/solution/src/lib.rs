use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ReviewSubmission {
    #[validate(length(min = 1, max = 200, message = "title must be 1-200 characters"))]
    pub title: String,

    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    pub rating: u8,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 1000, message = "comment must be at most 1000 characters"))]
    pub comment: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReviewError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("validation failed: {0:?}")]
    Invalid(Vec<String>),
}

pub fn validation_summary(errors: &ValidationErrors) -> Vec<String> {
    let mut messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                let message = e
                    .message
                    .clone()
                    .unwrap_or_else(|| e.code.clone())
                    .to_string();
                format!("{field}: {message}")
            })
        })
        .collect();
    messages.sort();
    messages
}

pub fn parse_review(json: &str) -> Result<ReviewSubmission, ReviewError> {
    let submission: ReviewSubmission =
        serde_json::from_str(json).map_err(|e| ReviewError::InvalidJson(e.to_string()))?;
    submission
        .validate()
        .map_err(|errors| ReviewError::Invalid(validation_summary(&errors)))?;
    Ok(submission)
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
