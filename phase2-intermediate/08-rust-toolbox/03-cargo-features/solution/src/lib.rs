/// Summary statistics for one batch of integer samples.
///
/// The struct exists in *both* builds; only the `serde::Serialize` derive
/// is feature-gated, via `cfg_attr`.
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
pub fn build_report(label: &str, samples: &[i64]) -> Option<Report> {
    let min = samples.iter().copied().min()?;
    let max = samples.iter().copied().max()?;
    let sum: i64 = samples.iter().sum();

    Some(Report {
        label: label.to_string(),
        count: samples.len(),
        mean: sum as f64 / samples.len() as f64,
        min,
        max,
    })
}

/// Serializes a [`Report`] to a JSON string. Only exists when the crate is
/// compiled with the `json-export` feature.
#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String {
    serde_json::to_string(report).expect("a plain struct of strings and numbers cannot fail to serialize")
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
//   cargo test --manifest-path .../solution/Cargo.toml --features json-export
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
