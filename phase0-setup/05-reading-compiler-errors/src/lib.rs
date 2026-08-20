//! Exercises for 05 — Reading compiler errors.
//!
//! Work through the ladder in the lesson's README. This file holds the
//! "implement" rung: three small functions, each fully specified in its doc
//! comment. You should never need to open the tests to know what to write.

/// Total items across `orders` orders that each contain `per_order` items.
///
/// # Examples
///
/// `total_items(7, 3)` returns `21`.
/// `total_items(0, 5)` returns `0`.
pub fn total_items(orders: u32, per_order: u32) -> u32 {
    todo!("return the two arguments multiplied together")
}

/// How many boxes are needed to pack `items`, where one box holds `per_box`
/// items. A partly-filled box still counts as a whole box.
///
/// `per_box` is never zero — the caller guarantees it.
///
/// # Examples
///
/// `boxes_needed(8, 4)` returns `2` — two full boxes.
/// `boxes_needed(10, 4)` returns `3` — two full boxes and one holding 2.
/// `boxes_needed(0, 4)` returns `0` — nothing to pack, no box needed.
pub fn boxes_needed(items: u32, per_box: u32) -> u32 {
    todo!("divide items by per_box, rounding up rather than down")
}

/// The number of **bytes** in `title`.
///
/// Bytes, not characters. For ASCII text the two are the same, and for text
/// like `"سلام"` they are not — Phase 1 has a whole lesson on why. For this
/// exercise you only need the byte count.
///
/// # Examples
///
/// `title_len("Frieren")` returns `7`.
/// `title_len("")` returns `0`.
pub fn title_len(title: &str) -> usize {
    todo!("return how many bytes `title` occupies")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_items() {
        assert_eq!(total_items(7, 3), 21);
        assert_eq!(total_items(0, 5), 0);
        assert_eq!(total_items(1, 1), 1);
    }

    #[test]
    fn rounds_boxes_up() {
        assert_eq!(boxes_needed(8, 4), 2);
        assert_eq!(boxes_needed(10, 4), 3);
        assert_eq!(boxes_needed(1, 4), 1);
        assert_eq!(boxes_needed(0, 4), 0);
    }

    #[test]
    fn counts_bytes_not_characters() {
        assert_eq!(title_len("Frieren"), 7);
        assert_eq!(title_len(""), 0);
        // Four Persian characters, eight bytes. Phase 1 explains this properly.
        assert_eq!(title_len("سلام"), 8);
    }
}
