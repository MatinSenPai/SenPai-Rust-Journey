//! The same two-variant enum in four JSON representations.
//! Run: `cargo run -p p3-03-01-serde-depth --example 03-tagged-enums`

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum External {
    Added { id: u64, title: String },
    Removed { id: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Internal {
    Added { id: u64, title: String },
    Removed { id: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Adjacent {
    Added { id: u64, title: String },
    Removed { id: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Added { id: u64, title: String },
    Removed { id: u64 },
}

fn main() {
    let t = || "Frieren".to_string();
    println!(
        "{}",
        serde_json::to_string(&External::Added { id: 1, title: t() }).unwrap()
    );
    println!(
        "{}",
        serde_json::to_string(&Internal::Added { id: 1, title: t() }).unwrap()
    );
    println!(
        "{}",
        serde_json::to_string(&Adjacent::Added { id: 1, title: t() }).unwrap()
    );
    println!(
        "{}",
        serde_json::to_string(&Untagged::Added { id: 1, title: t() }).unwrap()
    );

    println!(
        "{:?}",
        serde_json::from_str::<Internal>(r#"{"type":"Removed","id":1}"#)
    );
    println!(
        "{:?}",
        serde_json::from_str::<Internal>(r#"{"type":"Renamed","id":1}"#)
    );
    println!("{:?}", serde_json::from_str::<Internal>(r#"{"id":1}"#));
    println!("{:?}", serde_json::from_str::<Untagged>(r#"{"title":"x"}"#));
}
