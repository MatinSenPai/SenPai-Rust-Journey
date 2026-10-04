//! Compiles, runs, and quietly picks the wrong variant: `untagged` tries variants top to bottom.
//! Run: `cargo run -p p3-03-01-serde-depth --example 09-untagged-wrong-variant-trap`

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Input {
    Draft {
        title: String,
        rating: Option<u8>,
    },
    Full {
        title: String,
        rating: u8,
        status: String,
    },
}

fn main() {
    let body = r#"{"title":"Frieren","rating":9,"status":"watching"}"#;
    let parsed: Input = serde_json::from_str(body).unwrap();
    println!("{parsed:?}");
}
