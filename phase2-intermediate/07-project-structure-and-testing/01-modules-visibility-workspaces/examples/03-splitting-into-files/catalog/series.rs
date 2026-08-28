//! `series`'s own contents — one file, one module, exactly like `catalog.rs`
//! one level up. `shelf_code` is `pub(super)`: reachable from `catalog`
//! (this file's parent module), not from `main.rs` beyond it.

use super::Anime;

pub(super) fn shelf_code(a: &Anime) -> String {
    format!("SR-{}", a.title.len())
}
