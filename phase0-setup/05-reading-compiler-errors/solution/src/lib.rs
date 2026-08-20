//! Reference solution for 05 — Reading compiler errors.

/// Total items across `orders` orders that each contain `per_order` items.
pub fn total_items(orders: u32, per_order: u32) -> u32 {
    orders * per_order
}

/// How many boxes are needed to pack `items`, `per_box` items to a box.
pub fn boxes_needed(items: u32, per_box: u32) -> u32 {
    items.div_ceil(per_box)
}

/// The number of bytes in `title`.
pub fn title_len(title: &str) -> usize {
    title.len()
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
        assert_eq!(title_len("سلام"), 8);
    }
}
