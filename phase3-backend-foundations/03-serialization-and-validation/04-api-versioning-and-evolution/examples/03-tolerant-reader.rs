//! The same v2 body read by three clients. A tolerant reader ignores what it
//! does not know (serde's default) and defaults what is missing. A strict
//! reader (`deny_unknown_fields`) turns every additive change into an error.
//! The third reader survives a rename (`alias`) and a new enum value (`other`).
//!
//!     cargo run -p p3-03-04-api-versioning-and-evolution --example 03-tolerant-reader

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Tolerant {
    id: u64,
    title: String,
    #[serde(default)]
    episodes: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Strict {
    id: u64,
    title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Watching,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct Reader {
    #[serde(alias = "watch_status")]
    status: Status,
}

fn main() {
    let v2 = r#"{"id":1,"title":"Frieren","watch_status":"watching","episodes":28}"#;
    let old = r#"{"id":2,"title":"Dandadan"}"#;
    let t = |s| serde_json::from_str::<Tolerant>(s);
    let st = |s| serde_json::from_str::<Strict>(s);
    println!("tolerant, v2 body:  {:?}", t(v2));
    println!("tolerant, old body: {:?}", t(old));
    println!("strict,   v2 body:  {:?}", st(v2));
    println!("strict,   old body: {:?}", st(old));
    let r = |s| serde_json::from_str::<Reader>(s);
    println!("reader, old name:   {:?}", r(r#"{"status":"watching"}"#));
    println!(
        "reader, new name:   {:?}",
        r(r#"{"watch_status":"watching"}"#)
    );
    println!(
        "reader, new value:  {:?}",
        r(r#"{"watch_status":"on_hold"}"#)
    );
}
