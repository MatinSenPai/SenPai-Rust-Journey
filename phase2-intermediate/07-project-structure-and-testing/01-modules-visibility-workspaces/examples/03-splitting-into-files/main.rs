//! The exact same module tree as `02-pub-super-nested-visibility.rs` —
//! `catalog`, and `series` nested inside it — but now spread across real
//! files instead of one. `mod catalog;` (no body, just a semicolon) tells
//! Rust to look for the module's contents in a sibling file: `catalog.rs`,
//! right next to this one. Open it, then open `catalog/series.rs` from
//! inside it, and compare the tree to `02`'s — it's identical. Only the
//! file layout changed.

mod catalog;

fn main() {
    let a = catalog::Anime {
        title: "Mushishi".to_string(),
    };
    println!("{}", catalog::shelf_label(&a));
}
