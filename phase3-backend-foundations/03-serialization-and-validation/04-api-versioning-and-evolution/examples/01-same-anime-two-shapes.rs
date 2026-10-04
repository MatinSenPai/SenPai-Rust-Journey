//! One domain value, two wire shapes. `AnimeV1` is the frozen contract;
//! `AnimeV2` renames `status` and adds `episodes`. A `From` impl is the only
//! place the two vocabularies meet.
//!
//!     cargo run -p p3-03-04-api-versioning-and-evolution --example 01-same-anime-two-shapes

use serde::Serialize;

struct Anime {
    id: u64,
    title: String,
    status: String,
    rating: Option<u8>,
    episodes: Option<u32>,
}

#[derive(Serialize)]
struct AnimeV1 {
    id: u64,
    title: String,
    status: String,
    rating: Option<u8>,
}

#[derive(Serialize)]
struct AnimeV2 {
    id: u64,
    title: String,
    watch_status: String,
    rating: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    episodes: Option<u32>,
}

impl From<&Anime> for AnimeV1 {
    fn from(a: &Anime) -> Self {
        AnimeV1 {
            id: a.id,
            title: a.title.clone(),
            status: a.status.clone(),
            rating: a.rating,
        }
    }
}

impl From<&Anime> for AnimeV2 {
    fn from(a: &Anime) -> Self {
        AnimeV2 {
            id: a.id,
            title: a.title.clone(),
            watch_status: a.status.clone(),
            rating: a.rating,
            episodes: a.episodes,
        }
    }
}

fn main() {
    let list = [
        Anime {
            id: 1,
            title: "Frieren".into(),
            status: "watching".into(),
            rating: Some(9),
            episodes: Some(28),
        },
        Anime {
            id: 2,
            title: "Dandadan".into(),
            status: "plan_to_watch".into(),
            rating: None,
            episodes: None,
        },
    ];
    for a in &list {
        println!("v1 {}", serde_json::to_string(&AnimeV1::from(a)).unwrap());
        println!("v2 {}", serde_json::to_string(&AnimeV2::from(a)).unwrap());
    }
}
