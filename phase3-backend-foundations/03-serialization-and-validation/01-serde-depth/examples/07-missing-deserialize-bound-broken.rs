//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build -p p3-03-01-serde-depth --example 07-missing-deserialize-bound-broken --features broken`
//! and read the error.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Rating(u8);

#[derive(Debug, Serialize, Deserialize)]
struct Review {
    title: String,
    rating: Rating,
}

fn main() {
    let r: Review = serde_json::from_str(r#"{"title":"Frieren","rating":9}"#).unwrap();
    println!("{r:?}");
}
