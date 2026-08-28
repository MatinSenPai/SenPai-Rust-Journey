//! `cargo run -p p2-04-03-deref-asref-borrow --example 05-borrow-contract-violation`
//!
//! `CiKey` compiles fine and runs without panicking — that is what makes this
//! dangerous. Its `Hash`/`Eq` compare case-insensitively (lowercased), but
//! its `Borrow<str>` hands back the original, un-lowercased string. The two
//! disagree, and `HashMap` has no way to notice.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

struct CiKey(String);

impl PartialEq for CiKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
impl Eq for CiKey {}

impl Hash for CiKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

// Broken on purpose: borrows the ORIGINAL casing, so it disagrees with the
// lowercase-based Hash/Eq above.
impl Borrow<str> for CiKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

fn main() {
    let mut ratings: HashMap<CiKey, u8> = HashMap::new();
    ratings.insert(CiKey("Frieren".to_string()), 10);

    println!("len: {}", ratings.len());
    println!("get(\"Frieren\"): {:?}", ratings.get("Frieren"));
    println!("get(\"frieren\"): {:?}", ratings.get("frieren"));
}
