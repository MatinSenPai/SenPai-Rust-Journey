//! `HashMap::new()`, `.insert()`, and `.get()`/`.get_mut()` — the last two
//! return exactly the `Option` you already know from Phase 1, just aimed at a
//! lookup that might miss.
//!
//!     cargo run -p p2-01-02-hashmap-in-depth --example 01-new-insert-get

use std::collections::HashMap;

fn main() {
    let mut watched: HashMap<String, u32> = HashMap::new();

    watched.insert("Frieren".to_string(), 12);
    watched.insert("Bocchi the Rock".to_string(), 12);

    // `.insert()` returns the *previous* value, wrapped in `Option` — `None`
    // the first time a key is written, `Some(old)` if it overwrote one.
    let previous = watched.insert("Frieren".to_string(), 13);
    println!("previous episode count for Frieren: {previous:?}");

    // `.get()` never panics on a missing key — it hands back `Option<&V>`,
    // exactly the type 1.6.1 taught you to answer before touching what is
    // inside.
    match watched.get("Frieren") {
        Some(count) => println!("Frieren: {count} episodes"),
        None => println!("Frieren: not tracked"),
    }
    match watched.get("Attack on Titan") {
        Some(count) => println!("Attack on Titan: {count} episodes"),
        None => println!("Attack on Titan: not tracked"),
    }

    // `.get_mut()` is the mutable twin: `Option<&mut V>`.
    if let Some(count) = watched.get_mut("Bocchi the Rock") {
        *count += 1;
    }
    println!("Bocchi the Rock: {:?}", watched.get("Bocchi the Rock"));

    // `map[key]` also works, but it panics on a missing key instead of
    // handing you an `Option` — reach for `.get()` unless you have already
    // proven the key is there.
    println!("indexing a key you know exists: {}", watched["Frieren"]);
}
