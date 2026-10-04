//! `rename_all`, `skip_serializing_if` and `default` on one struct.
//! Run: `cargo run -p p3-03-01-serde-depth --example 01-rename-skip-default`

use p3_03_01_serde_depth::WatchStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Card {
    id: u64,
    title: String,
    watch_status: WatchStatus,
    #[serde(default)]
    episode_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<u8>,
}

fn main() {
    let mut card = Card {
        id: 1,
        title: "Frieren".into(),
        watch_status: WatchStatus::Watching,
        episode_count: 28,
        rating: Some(9),
    };
    println!("{}", serde_json::to_string(&card).unwrap());
    card.rating = None;
    println!("{}", serde_json::to_string(&card).unwrap());

    let input = r#"{"id":2,"title":"Mushishi","watchStatus":"completed"}"#;
    println!("{:?}", serde_json::from_str::<Card>(input));

    let snake = r#"{"id":2,"title":"Mushishi","watch_status":"completed"}"#;
    println!("{:?}", serde_json::from_str::<Card>(snake));
}
