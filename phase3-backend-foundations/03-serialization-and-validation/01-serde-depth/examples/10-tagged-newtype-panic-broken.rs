//! DELIBERATELY BROKEN — expected: a run-time panic (an internal tag cannot sit next to a bare integer)
//! Run `cargo run -p p3-03-01-serde-depth --example 10-tagged-newtype-panic-broken --features broken`
//! and read the panic.

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
enum Lookup {
    ById(u64),
    ByTitle { title: String },
}

fn main() {
    let ok = serde_json::to_string(&Lookup::ByTitle {
        title: "Frieren".into(),
    })
    .unwrap();
    println!("{ok}");
    let bad = serde_json::to_string(&Lookup::ById(1)).unwrap();
    println!("{bad}");
}
