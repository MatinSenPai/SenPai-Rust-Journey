//! Challenge: diffing two response shapes.

use p3_03_04_api_versioning_and_evolution_solution::{
    diff_shapes, requires_new_version, Change::*,
};
use serde_json::json;

#[test]
fn v1_to_v2_is_a_rename_plus_an_addition() {
    let v1 = json!({"id":1,"title":"x","status":"watching","rating":9});
    let v2 = json!({"id":1,"title":"x","watch_status":"watching","rating":9,"episodes":28});
    let d = diff_shapes(&v1, &v2);
    assert_eq!(
        d,
        vec![RemoveResponseField, AddResponseField, AddResponseField]
    );
    assert!(requires_new_version(&d));
}

#[test]
fn a_type_change_is_reported() {
    let d = diff_shapes(&json!({"rating": 9}), &json!({"rating": "9"}));
    assert_eq!(d, vec![ChangeResponseFieldType]);
}

#[test]
fn identical_shapes_and_non_objects_give_nothing() {
    assert!(diff_shapes(&json!({"a":1}), &json!({"a":2})).is_empty());
    assert!(diff_shapes(&json!([1]), &json!({"a":1})).is_empty());
}

#[test]
fn only_additions_do_not_need_a_new_version() {
    let d = diff_shapes(&json!({"a":1}), &json!({"a":1,"b":2}));
    assert_eq!(d, vec![AddResponseField]);
    assert!(!requires_new_version(&d));
}
