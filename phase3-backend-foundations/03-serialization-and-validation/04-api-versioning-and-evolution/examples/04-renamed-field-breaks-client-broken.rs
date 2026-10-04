//! DELIBERATELY BROKEN — expected: a run-time panic (missing field `status`)
//!
//! A client written against v1 reads a v2 body. The server renamed `status` to
//! `watch_status`, so the old client's `unwrap` panics. Nothing in the
//! client's code changed; the contract under it did.
//!
//!     cargo run -p p3-03-04-api-versioning-and-evolution --example 04-renamed-field-breaks-client-broken --features broken

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnimeV1 {
    id: u64,
    status: String,
}

fn main() {
    let body = r#"{"id":1,"watch_status":"watching","episodes":28}"#;
    let anime: AnimeV1 = serde_json::from_str(body).unwrap();
    println!("{anime:?}");
}
