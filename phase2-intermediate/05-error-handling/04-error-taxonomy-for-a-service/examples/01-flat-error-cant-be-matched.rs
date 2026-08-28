//! The problem this lesson solves: a flat `Result<T, String>` gives a caller
//! nothing to act on except the text itself.
//!
//!     cargo run -p p2-05-04-error-taxonomy-for-a-service --example 01-flat-error-cant-be-matched

fn add_entry_stringly(title: &str, rating: u8) -> Result<u64, String> {
    if title.trim().is_empty() {
        return Err("title must not be empty".to_string());
    }
    if rating > 10 {
        return Err(format!("rating {rating} is out of range 0..=10"));
    }
    Ok(0)
}

fn main() {
    match add_entry_stringly("", 5) {
        Ok(id) => println!("added as {id}"),
        Err(msg) if msg.contains("empty") => {
            println!("caller reaction: ask again for a title ({msg})");
        }
        Err(msg) if msg.contains("out of range") => {
            println!("caller reaction: ask again for a rating ({msg})");
        }
        Err(msg) => println!("caller reaction: unknown failure: {msg}"),
    }
}
