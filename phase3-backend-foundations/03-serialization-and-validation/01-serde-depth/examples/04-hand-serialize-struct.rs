//! A hand-written `Serialize` for a struct: a computed field and a skipped one.
//! Run: `cargo run -p p3-03-01-serde-depth --example 04-hand-serialize-struct`

use serde::ser::{Serialize, SerializeStruct, Serializer};

struct Anime {
    id: u64,
    title: String,
    rating: Option<u8>,
}

impl Serialize for Anime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = if self.rating.is_some() { 4 } else { 3 };
        let mut s = serializer.serialize_struct("Anime", len)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("title", &self.title)?;
        if let Some(r) = self.rating {
            s.serialize_field("rating", &r)?;
        }
        s.serialize_field("label", &format!("#{} {}", self.id, self.title))?;
        s.end()
    }
}

fn main() {
    let a = Anime {
        id: 1,
        title: "Frieren".into(),
        rating: Some(9),
    };
    println!("{}", serde_json::to_string(&a).unwrap());
    let b = Anime {
        id: 2,
        title: "Mushishi".into(),
        rating: None,
    };
    println!("{}", serde_json::to_string(&b).unwrap());
}
