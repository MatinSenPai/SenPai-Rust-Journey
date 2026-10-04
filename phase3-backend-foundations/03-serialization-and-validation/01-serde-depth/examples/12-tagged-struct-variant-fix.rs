//! The fix for example 10: an internally tagged enum needs struct variants.
//! Run: `cargo run -p p3-03-01-serde-depth --example 12-tagged-struct-variant-fix`

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Lookup {
    ById { id: u64 },
    ByTitle { title: String },
}

fn main() {
    let by_id = serde_json::to_string(&Lookup::ById { id: 1 }).unwrap();
    let by_title = Lookup::ByTitle {
        title: "Frieren".into(),
    };
    println!("{by_id}");
    println!("{}", serde_json::to_string(&by_title).unwrap());
}
