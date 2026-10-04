//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build -p p3-03-01-serde-depth --example 08-default-without-default-broken --features broken`
//! and read the error.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
enum Status {
    Watching,
    Dropped,
}

#[derive(Debug, Deserialize)]
struct Create {
    title: String,
    #[serde(default)]
    status: Status,
}

fn main() {
    let c: Create = serde_json::from_str(r#"{"title":"Frieren"}"#).unwrap();
    println!("{c:?}");
}
