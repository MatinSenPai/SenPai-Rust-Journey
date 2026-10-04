//! `flatten`: merging two structs into one JSON object, and a catch-all map.
//! Run: `cargo run -p p3-03-01-serde-depth --example 02-flatten`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
struct Card {
    id: u64,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Audit {
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Detail {
    #[serde(flatten)]
    card: Card,
    #[serde(flatten)]
    audit: Audit,
}

#[derive(Debug, Serialize, Deserialize)]
struct WithExtras {
    id: u64,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

fn main() {
    let detail = Detail {
        card: Card {
            id: 1,
            title: "Frieren".into(),
        },
        audit: Audit {
            created_at: "2026-01-05".into(),
        },
    };
    let json = serde_json::to_string(&detail).unwrap();
    println!("{json}");
    println!("{:?}", serde_json::from_str::<Detail>(&json).unwrap());

    let loose = r#"{"id":7,"studio":"Madhouse","year":2023}"#;
    println!("{:?}", serde_json::from_str::<WithExtras>(loose).unwrap());
}
