/// Summary statistics for one batch of integer samples.
///
/// The `cfg_attr` line is the lesson: the struct exists in *both* builds,
/// but it only derives `serde::Serialize` when the `json-export` feature
/// is enabled — a plain `#[cfg(feature = "json-export")]` here would make
/// the whole struct vanish from the default build instead.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json-export", derive(serde::Serialize))]
pub struct Report {
    pub label: String,
    pub count: usize,
    pub mean: f64,
    pub min: i64,
    pub max: i64,
}

/// Builds a [`Report`] over `samples`, or `None` if the slice is empty.
///
/// Always available — no feature gate. `label` becomes `Report::label` as an
/// owned `String`. `count` is `samples.len()`. `mean` is the sum of
/// `samples` divided by `count`, as an `f64`. `min` and `max` are the
/// smallest and largest values in `samples`. On an empty slice, every field
/// is undefined — return `None` instead of computing anything.
pub fn build_report(label: &str, samples: &[i64]) -> Option<Report> {
    todo!("compute label, count, mean, min and max as described above; None on an empty slice")
}

/// Serializes a [`Report`] to a JSON string, one key per field, under the
/// field's own name (`label`, `count`, `mean`, `min`, `max`).
///
/// This function only exists when the crate is compiled with the
/// `json-export` feature — in the default build it is removed *before*
/// type checking, which is why the crate needs no serde at all by default.
/// A `Report` of plain strings and numbers cannot fail to serialize, so
/// treat that failure as a bug, not a value to hand back to the caller.
#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String {
    todo!("serialize report to a JSON string with serde_json; panic if that ever fails")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_report_computes_all_five_fields() {
        let report = build_report("api latency", &[10, 20, 60]).unwrap();
        assert_eq!(report.label, "api latency");
        assert_eq!(report.count, 3);
        assert!((report.mean - 30.0).abs() < f64::EPSILON);
        assert_eq!(report.min, 10);
        assert_eq!(report.max, 60);
    }

    #[test]
    fn build_report_handles_negative_samples() {
        let report = build_report("deltas", &[-5, 5]).unwrap();
        assert_eq!(report.min, -5);
        assert_eq!(report.max, 5);
        assert!((report.mean - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_report_of_empty_samples_is_none() {
        assert!(build_report("empty", &[]).is_none());
    }
}

// These tests only compile — let alone run — when the feature is on:
//   cargo test -p p2-10-03-cargo-features --features json-export
#[cfg(all(test, feature = "json-export"))]
mod json_tests {
    use super::*;

    #[test]
    fn to_json_produces_parseable_json_with_all_fields() {
        let report = build_report("api latency", &[10, 20, 60]).unwrap();
        let json = to_json(&report);

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["label"], "api latency");
        assert_eq!(value["count"], 3);
        assert_eq!(value["mean"], 30.0);
        assert_eq!(value["min"], 10);
        assert_eq!(value["max"], 60);
    }
}
