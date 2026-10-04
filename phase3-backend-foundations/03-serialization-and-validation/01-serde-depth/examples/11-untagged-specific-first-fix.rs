//! The fix for example 09: put the most specific untagged variant first.
//! Run: `cargo run -p p3-03-01-serde-depth --example 11-untagged-specific-first-fix`

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Input {
    Full {
        title: String,
        rating: u8,
        status: String,
    },
    Draft {
        title: String,
        rating: Option<u8>,
    },
}

fn main() {
    let full = r#"{"title":"Frieren","rating":9,"status":"watching"}"#;
    let draft = r#"{"title":"Frieren"}"#;
    println!("{:?}", serde_json::from_str::<Input>(full).unwrap());
    println!("{:?}", serde_json::from_str::<Input>(draft).unwrap());
}
