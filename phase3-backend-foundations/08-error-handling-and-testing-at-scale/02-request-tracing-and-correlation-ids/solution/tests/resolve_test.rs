use p3_08_02_request_tracing_and_correlation_ids_solution::{
    error_body, is_valid_request_id, new_request_id, resolve_request_id,
};
use serde_json::json;

#[test]
fn accepts_ordinary_ids() {
    assert!(is_valid_request_id("abc-123"));
    assert!(is_valid_request_id("A_b.C-9"));
    assert!(is_valid_request_id("x"));
    assert!(is_valid_request_id("67e55044-10b1-426f-9247-bb680e5fe0c8"));
}

#[test]
fn length_limit_is_64_bytes_inclusive() {
    assert!(is_valid_request_id(&"a".repeat(64)));
    assert!(!is_valid_request_id(&"a".repeat(65)));
    assert!(!is_valid_request_id(""));
}

#[test]
fn rejects_characters_outside_the_allowed_set() {
    for bad in [
        "has space",
        "quo\"te",
        "a/b",
        "a=b",
        "caf\u{e9}",
        "tab\there",
        "a,b",
    ] {
        assert!(!is_valid_request_id(bad), "{bad:?} should be rejected");
    }
}

#[test]
fn valid_incoming_id_is_kept_and_generate_is_not_called() {
    let id = resolve_request_id(Some("abc-123"), || panic!("generate must not run"));
    assert_eq!(id, "abc-123");
}

#[test]
fn missing_incoming_id_is_generated() {
    assert_eq!(resolve_request_id(None, || "fresh".to_string()), "fresh");
}

#[test]
fn invalid_incoming_id_is_replaced_not_repaired() {
    let long = "a".repeat(200);
    for bad in ["has space", "", long.as_str()] {
        assert_eq!(
            resolve_request_id(Some(bad), || "fresh".to_string()),
            "fresh"
        );
    }
}

#[test]
fn generated_ids_are_uuids_and_unique() {
    let a = new_request_id();
    assert_eq!(a.len(), 36);
    assert!(is_valid_request_id(&a));
    assert_ne!(a, new_request_id());
}

#[test]
fn error_body_has_the_documented_shape() {
    assert_eq!(
        error_body("abc-123", "not_found", "anime 7 not found"),
        json!({"error": {
            "code": "not_found",
            "message": "anime 7 not found",
            "request_id": "abc-123",
        }})
    );
}
