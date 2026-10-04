//! A typo'd key: silently dropped by default, rejected with `deny_unknown_fields`.
//! Also: `deny_unknown_fields` next to `flatten` is documented as unsupported; see what it does.
//! Run: `cargo run -p p3-03-01-serde-depth --example 06-deny-unknown-fields`

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct Lenient {
    title: String,
    rating: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strict {
    title: String,
    rating: Option<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Contradiction {
    title: String,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

fn main() {
    let typo = r#"{"title":"Frieren","ratng":9}"#;
    println!("{:?}", serde_json::from_str::<Lenient>(typo));
    println!("{:?}", serde_json::from_str::<Strict>(typo));
    println!("{:?}", serde_json::from_str::<Contradiction>(typo));
}
