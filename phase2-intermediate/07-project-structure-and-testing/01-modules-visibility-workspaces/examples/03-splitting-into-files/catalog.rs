//! The contents of the `catalog` module — this file exists because
//! `main.rs` wrote `mod catalog;`. In turn, `mod series;` below means Rust
//! looks for `series`'s contents in `catalog/series.rs`: a subdirectory
//! named after this module, sitting next to this file.

pub mod series;

pub struct Anime {
    pub title: String,
}

pub fn shelf_label(a: &Anime) -> String {
    format!("{}: {}", a.title, series::shelf_code(a))
}
